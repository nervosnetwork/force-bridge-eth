pragma solidity ^0.5.10;
import {TypedMemView} from "./TypedMemView.sol";
import {ViewCKB} from "./ViewCKB.sol";
import {SafeMath} from "./SafeMath.sol";

library ViewSpv {
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using SafeMath for uint;

    enum SpvTypes {
        Unknown,                // 0x0
        TransactionProof,
        WitnessesRoot,
        JsonMerkleProof
    }

    /// @notice             requires `memView` to be of a specified type
    /// @param memView      a 29-byte view with a 5-byte type
    /// @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    /// @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, SpvTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    function blockHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.TransactionProof) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return _input.index(startIndex, 32);
    }

    function witnessedRoot(bytes29 _input) internal pure typeAssert(_input, SpvTypes.WitnessesRoot) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return _input.index(startIndex, 32);
    }

    function mockTxHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.TransactionProof) returns (bytes32) {
        return bytes32(0);
    }
}
