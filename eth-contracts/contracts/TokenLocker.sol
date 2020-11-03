pragma solidity ^0.5.10;

import "./interfaces/IERC20.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {CKBTxView} from "./libraries/CKBTxView.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";

contract TokenLocker {
    using SafeMath for uint256;
    using Address for address;

    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using CKBTxView for bytes29;
    using ViewSpv for bytes29;

    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;
    bytes32 public recipientCellTypeCodeHash_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;

    event Locked(
        address indexed token,
        address indexed sender,
        uint256 amount,
        string ckbAddress,
        string replayResistOutpoint
    );

    event Unlocked(
        address indexed token,
        uint256 amount,
        address recipient
    );

    // Function output from burning fungible token on Near side.
    struct BurnResult {
        uint256 amount;
        address token;
        address recipient;
    }

    constructor(address ckbSpvAddress, uint64 numConfirmations, bytes32 recipientCellTypeCodeHash) public {
        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
        recipientCellTypeCodeHash_ = recipientCellTypeCodeHash;
    }

    function _decodeBurnResult(bytes memory ckbTx) internal view returns (BurnResult memory) {
        // TODO
        // 1. check if ckbTx matches  ckbTxProof.txHash
        // 2. _decodeBurnResult from ckbTx
//        uint128 mockAmount = 111100000000000000;
//        address mockToken = address(0);
//        address mockRecipient = address(0);
//        result = BurnResult(mockAmount, mockToken, mockRecipient);

        bytes29 rawTx = ckbTx.ref(uint40(CKBTxView.CKBTxTypes.RawTx));
        bytes29 recipientCellTypescript = rawTx.outputs().recipientCellOutput().typescript();
        require((recipientCellTypescript.codeHash() == recipientCellTypeCodeHash_), "invalid recipient cell typescript code hash");
        require((recipientCellTypescript.hashType() == 0), "invalid recipient cell typescript hash type");
        require((recipientCellTypescript.args() == address(this)), "invalid recipient cell typescript hash type");
        bytes29 recipientCellData = rawTx.outputsData().recipientCellData();
        return BurnResult(recipientCellData.tokenAmount(), recipientCellData.tokenAddress(), recipientCellData.recipientAddress());
    }

    // before lockToken, user should approve -> TokenLocker Contract with 0xffffff token
    function lockToken(address token, uint256 amount, string memory ckbAddress, string memory replayResistOutpoint) public {
        // TODO modify `transferFrom` to `safeTransferFrom`
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Locked(token, msg.sender, amount, ckbAddress, replayResistOutpoint);
    }

    function lockETH(string memory ckbAddress, string memory replayResistOutpoint) public payable {
        emit Locked(address(0), msg.sender, msg.value, ckbAddress, replayResistOutpoint);
    }

    function unlockToken(bytes memory ckbTxProof, bytes memory ckbTx) public {
        require(ckbSpv_.proveTxExist(ckbTxProof, numConfirmations_), "tx from proofData should exist");

        // Unpack the proof and extract the execution outcome.
        bytes29 proof = ckbTxProof.ref(uint40(ViewSpv.SpvTypes.CKBTxProof));

        // TODO modify `mockTxHash` to `txHash`
        bytes32 txHash = proof.txHash();
        require(!usedTx_[txHash], "The burn tx cannot be reused");
        usedTx_[txHash] = true;
        require((txHash == CKBCrypto.digest(ckbTx, ckbTx.length)), "ckbTx mismatched with ckbTxProof");

        BurnResult memory result = _decodeBurnResult(ckbTx);
        // TODO modify `transfer` to `safeTransfer`
        if (result.token == address(0)) {
            // it means token == Eth
            result.recipient.toPayable().transfer(result.amount);
        } else {
            IERC20(result.token).transfer(result.recipient, result.amount);
        }

        emit Unlocked(address(result.token), result.amount, result.recipient);
    }
}
