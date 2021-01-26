// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {CKBCrypto} from "../libraries/CKBCrypto.sol";
import {TypedMemView} from "../libraries/TypedMemView.sol";
import {SafeMath} from "../libraries/SafeMath.sol";
import {CKBTxView} from "../libraries/CKBTxView.sol";
import {ViewSpv} from "../libraries/ViewSpv.sol";
import {Address} from "../libraries/Address.sol";
import {SafeERC20} from "../libraries/SafeERC20.sol";
import {IERC20} from "../interfaces/IERC20.sol";
import {ICKBSpv} from "../interfaces/ICKBSpv.sol";

contract TokenLocker {
    using SafeMath for uint256;
    using Address for address;
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using CKBTxView for bytes29;
    using ViewSpv for bytes29;
    using SafeERC20 for IERC20;

    bool public initialized;
    uint8 public recipientCellTypescriptHashType_;
    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;
    bytes32 public recipientCellTypescriptCodeHash_;
    bytes32 public lightClientTypescriptHash_;
    bytes32 public bridgeCellLockscriptCodeHash_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;

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

    function initialize(
        address ckbSpvAddress,
        uint64 numConfirmations,
        bytes32 recipientCellTypescriptCodeHash,
        uint8 typescriptHashType,
        bytes32 lightClientTypescriptHash,
        bytes32 bridgeCellLockscriptCodeHash
    ) public {
        require(!initialized, "Contract instance has already been initialized");
        initialized = true;

        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
        recipientCellTypescriptCodeHash_ = recipientCellTypescriptCodeHash;
        recipientCellTypescriptHashType_ = typescriptHashType;
        lightClientTypescriptHash_ = lightClientTypescriptHash;
        bridgeCellLockscriptCodeHash_ = bridgeCellLockscriptCodeHash;
    }

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
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
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

        // if token == ETH
        if (tokenAddress == address(0)) {
            payable(recipientAddress).transfer(receivedAmount);
            payable(msg.sender).transfer(bridgeFee);
        } else {
            IERC20(tokenAddress).safeTransfer(recipientAddress, receivedAmount);
            IERC20(tokenAddress).safeTransfer(msg.sender, bridgeFee);
        }

        emit Unlocked(tokenAddress, recipientAddress, msg.sender, receivedAmount, bridgeFee);
    }

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

    function setLightClientTypescriptHash() public {
        lightClientTypescriptHash_ = 0xbb867b58869bdcd636c2c1d0256dd087630ac88619dda33537f887889ddaa233;
    }
}


