// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./interfaces/IERC20.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {CKBTxView} from "./libraries/CKBTxView.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import {MultisigUtils} from "./libraries/MultisigUtils.sol";
import "./TokenLockerLayout.sol";
import "./proxy/Delegate.sol";

contract TokenLockerLogic is Delegate, TokenLockerLayout {

    using SafeMath for uint256;
    using Address for address;
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using CKBTxView for bytes29;
    using ViewSpv for bytes29;

    event Locked(
        address indexed token,
        address indexed sender,
        uint256 lockedAmount,
        uint256 bridgeFee,
        bytes recipientLockscript,
        bytes replayResistOutpoint,
        bytes sudtExtraData
    );

    event Unlocked(
        address indexed token,
        address indexed recipient,
        address indexed sender,
        uint256 receivedAmount,
        uint256 bridgeFee
    );

    function lockETH(
        uint256 bridgeFee,
        bytes memory recipientLockscript,
        bytes memory replayResistOutpoint,
        bytes memory sudtExtraData
    ) public payable {
        require(msg.value > bridgeFee, "fee should not exceed bridge amount");
        emit Locked(
            address(0),
            msg.sender,
            msg.value,
            bridgeFee,
            recipientLockscript,
            replayResistOutpoint,
            sudtExtraData
        );
    }

    // before lockToken, user should approve -> TokenLocker Contract with 0xffffff token
    function lockToken(
        address token,
        uint256 amount,
        uint256 bridgeFee,
        bytes memory recipientLockscript,
        bytes memory replayResistOutpoint,
        bytes memory sudtExtraData
    ) public {
        require(amount > bridgeFee, "fee should not exceed bridge amount");
        // TODO modify `transferFrom` to `safeTransferFrom`
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        emit Locked(
            token,
            msg.sender,
            amount,
            bridgeFee,
            recipientLockscript,
            replayResistOutpoint,
            sudtExtraData
        );
    }

    function unlockToken(bytes memory ckbTxProof, bytes memory ckbTx) public {
        require(ckbSpv_.proveTxExist(ckbTxProof, numConfirmations_), "tx from proofData should exist");

        bytes29 proof = ckbTxProof.ref(uint40(ViewSpv.SpvTypes.CKBTxProof));
        bytes32 txHash = proof.txHash();
        require(!usedTx_[txHash], "The burn tx cannot be reused");
        usedTx_[txHash] = true;
        require((txHash == CKBCrypto.digest(ckbTx, ckbTx.length)), "ckbTx mismatched with ckbTxProof");

        (uint256 bridgeAmount, uint256 bridgeFee, address tokenAddress, address recipientAddress) = decodeBurnResult(ckbTx);
        require(bridgeAmount > bridgeFee, "fee should not exceed bridge amount");
        uint256 receivedAmount = bridgeAmount - bridgeFee;

        // TODO modify `transfer` to `safeTransfer`
        // if token == ETH
        if (tokenAddress == address(0)) {
            recipientAddress.toPayable().transfer(receivedAmount);
            payable(msg.sender).transfer(bridgeFee);
        } else {
            IERC20(tokenAddress).transfer(recipientAddress, receivedAmount);
            IERC20(tokenAddress).transfer(msg.sender, bridgeFee);
        }

        emit Unlocked(tokenAddress, recipientAddress, msg.sender, receivedAmount, bridgeFee);
    }

    // TODO refund function
    function decodeBurnResult(bytes memory ckbTx) public view returns (
        uint256 bridgeAmount,
        uint256 bridgeFee,
        address token,
        address recipient
    ){
        bytes29 rawTx = ckbTx.ref(uint40(CKBTxView.CKBTxTypes.RawTx));
        bytes29 recipientCellTypescript = rawTx.outputs().recipientCellOutput().typescript();
        require(
            (recipientCellTypescript.recipientTypescriptCodeHash() == recipientCellTypescriptCodeHash_),
            "invalid recipient cell typescript code hash"
        );
        require((recipientCellTypescript.hashType() == recipientCellTypescriptHashType_), "invalid recipient cell typescript hash type");
        bytes29 recipientCellData = rawTx.outputsData().recipientCellData();
        require((recipientCellData.contractAddress() == address(this)), "invalid contract address in recipient cell");
        require((recipientCellData.lightClientTypescriptHash() == lightClientTypescriptHash_), "invalid lightClientTypescriptHash in recipient cell");
        require((recipientCellData.bridgeLockscriptCodeHash() == bridgeCellLockscriptCodeHash_), "invalid bridgeLockscriptCodeHash in recipient cell");
        return (
        recipientCellData.bridgeAmount(),
        recipientCellData.bridgeFee(),
        recipientCellData.tokenAddress(),
        recipientCellData.recipientAddress()
        );
    }
}
