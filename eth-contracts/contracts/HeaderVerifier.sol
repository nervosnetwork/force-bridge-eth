pragma solidity ^0.5.10;
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {EaglesongLib} from "./libraries/EaglesongLib.sol";
import {ICKBChain} from "./interfaces/ICKBChain.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import "hardhat/console.sol";

contract HeaderVerifier {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using ViewSpv for bytes29;
    uint256 constant public MAX_UIN256 = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

    /// @notice             requires `memView` to be of a specified type
    /// @param memView      a 29-byte view with a 5-byte type
    /// @param t            the expected type (e.g. BTCTypes.Outpoint, BTCTypes.TxIn, etc)
    /// @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, ViewCKB.CKBTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    function verifyHeader(bytes calldata input) external returns (bool) {
        bytes29 headerView = input.ref(uint40(ViewCKB.CKBTypes.Header));
        require(_verifyHeader(headerView), "expect header verified");
        return _verifyHeader(headerView);
    }

    /// #Verify header
    function _verifyHeader(bytes29 headerView) public typeAssert(headerView, ViewCKB.CKBTypes.Header) returns (bool) {
        // TODO verify header's pow and version
        bytes29 rawHeader = headerView.rawHeader();
        uint64 blockNumber = rawHeader.blockNumber();

        // verify pow
        // - 1. same epoch
        // calc input = pow_message(header..calc_pow_hash(), nonce)
        bytes memory input = _powMessage(headerView);

        // calc output by eaglesong(&input, &mut output);
        bytes memory output = EaglesongLib.EaglesongHash(input);
        bytes memory expectOutput= hex"000000000000053ee598839a89638a5b37a7cf98ecf0ce6d02d3d9287f008b84";
        require(keccak256(output) == keccak256(expectOutput), "eaglesong error");

        // calc block_target
        (uint256 target, bool overflow) = _compactToTarget(rawHeader.compactTarget());
        require(target == 13919424058362656885362395578858131467813097398447086657077248, "target error");
        require(!overflow, "overflow error");
        if (target == 0 || overflow) {
            return false;
        }

        // require( U256::from_big_endian(&output[..]) <= block_target )
        require(uint256(output.ref(uint40(ViewCKB.CKBTypes.H256)).indexUint(0, 32)) == 8429500200497070941028840761414215739490212483312848713321348, "000111");
        if (uint256(output.ref(uint40(ViewCKB.CKBTypes.H256)).indexUint(0, 32)) > target) {
            return false;
        }
        return true;
    }

    function _powMessage(bytes29 headerView) internal view returns (bytes memory) {
        // the 64 == 256 - rawHeader.len()
        bytes32 rawHeaderHash = CKBCrypto.digest(abi.encodePacked(headerView.rawHeader().clone(), new bytes(64)), 192);
        return abi.encodePacked(rawHeaderHash, headerView.slice(192, 16, uint40(ViewCKB.CKBTypes.Nonce)).clone());
    }

    function _compactToTarget(uint32 compact) internal pure returns (uint256, bool) {
        /*
            let exponent = compact >> 24;
            let mut mantissa = U256::from(compact & 0x00ff_ffff);

            let mut ret;
            if exponent <= 3 {
                mantissa >>= 8 * (3 - exponent);
                ret = mantissa.clone();
            } else {
                ret = mantissa.clone();
                ret <<= 8 * (exponent - 3);
            }

            let overflow = !mantissa.is_zero() && (exponent > 32);
            (ret, overflow)
        */

        uint32 exponent = compact >> 24;
        uint256 mantissa = uint256(compact & 0x00ff_ffff);
        uint256 ret;
        if (exponent <= 3) {
            mantissa >>= 8 * (3 - exponent);
            ret = mantissa;
        } else {
            ret = mantissa;
            ret <<= 8 * (exponent - 3);
        }
        bool overflow =  mantissa != 0 && (exponent > 32);
        return (ret, overflow);
    }

    function _targetToDifficulty(uint256 target) internal pure returns (uint256) {
        if (target == 1) {
            return MAX_UIN256;
        }

        if (MAX_UIN256 % target == target - 1) {
            return MAX_UIN256 / target + 1;
        }

        return MAX_UIN256 / target;
    }
}
