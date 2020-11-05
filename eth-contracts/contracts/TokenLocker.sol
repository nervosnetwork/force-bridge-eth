pragma solidity ^0.5.10;
pragma experimental ABIEncoderV2;

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

    struct BurnResult {
        uint256 amount;
        address token;
        address recipient;
    }

    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;
    bytes32 public recipientCellTypescriptCodeHash_;
    uint8 public recipientCellTypescriptHashType_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;

    event Locked(
        address indexed token,
        address indexed sender,
        string  indexed ckbAddress,
        uint256 amount,
        string replayResistOutpoint
    );

    event Unlocked(
        address indexed token,
        address indexed recipient,
        uint256 amount
    );

    constructor(address ckbSpvAddress, uint64 numConfirmations, bytes32 typescriptCodeHash, uint8 typescriptHashType) public {
        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
        recipientCellTypescriptCodeHash_ = typescriptCodeHash;
        recipientCellTypescriptHashType_ = typescriptHashType;
    }

    // before lockToken, user should approve -> TokenLocker Contract with 0xffffff token
    function lockToken(address token, uint256 amount, string memory ckbAddress, string memory replayResistOutpoint) public {
        // TODO modify `transferFrom` to `safeTransferFrom`
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Locked(token, msg.sender, ckbAddress, amount, replayResistOutpoint);
    }

    function lockETH(string memory ckbAddress, string memory replayResistOutpoint) public payable {
        emit Locked(address(0), msg.sender, ckbAddress, msg.value, replayResistOutpoint);
    }

    function unlockToken(bytes memory ckbTxProof, bytes memory ckbTx) public {
        require(ckbSpv_.proveTxExist(ckbTxProof, numConfirmations_), "tx from proofData should exist");

        bytes29 proof = ckbTxProof.ref(uint40(ViewSpv.SpvTypes.CKBTxProof));
        bytes32 txHash = proof.txHash();
        require(!usedTx_[txHash], "The burn tx cannot be reused");
        usedTx_[txHash] = true;
        require((txHash == CKBCrypto.digest(ckbTx, ckbTx.length)), "ckbTx mismatched with ckbTxProof");

        BurnResult memory result = decodeBurnResult(ckbTx);
        // TODO modify `transfer` to `safeTransfer`
        // if token == ETH
        if (result.token == address(0)) {
            result.recipient.toPayable().transfer(result.amount);
        } else {
            IERC20(result.token).transfer(result.recipient, result.amount);
        }

        emit Unlocked(address(result.token), result.recipient, result.amount);
    }

    function decodeBurnResult(bytes memory ckbTx) public view returns (BurnResult memory result) {
        bytes29 rawTx = ckbTx.ref(uint40(CKBTxView.CKBTxTypes.RawTx));
        bytes29 recipientCellTypescript = rawTx.outputs().recipientCellOutput().typescript();
        require((recipientCellTypescript.codeHash() == recipientCellTypescriptCodeHash_), "invalid recipient cell typescript code hash");
        require((recipientCellTypescript.hashType() == 0), "invalid recipient cell typescript hash type");
        require((recipientCellTypescript.args() == address(this)), "invalid recipient cell typescript args");
        bytes29 recipientCellData = rawTx.outputsData().recipientCellData();
        result = BurnResult(recipientCellData.tokenAmount(), recipientCellData.tokenAddress(), recipientCellData.recipientAddress());
    }
}
