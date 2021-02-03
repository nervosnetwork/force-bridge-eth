// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {CKBTxView} from "./libraries/CKBTxView.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {SafeERC20} from "./libraries/SafeERC20.sol";
import {IERC20} from "./interfaces/IERC20.sol";
import {ICKBSpvV3} from "./interfaces/ICKBSpvV3.sol";
import {Blake2b} from "./libraries/Blake2b.sol";

contract TokenLockerV2 {
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
    ICKBSpvV3 public ckbSpv_;
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

        ckbSpv_ = ICKBSpvV3(ckbSpvAddress);
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

    function unlockToken(bytes[] calldata ckbTxProofArray, bytes[] calldata ckbTxs, bytes calldata txRootProofData) external {
        // 1. check proveTxRootExist
        require(ckbSpv_.proveTxRootExist(txRootProofData), "txRoot from txRootProofData should exist on CKBChain");

        // 2. check raw ckbTx, txProof and txRootProof match
        bytes29 txRootProofView = txRootProofData.ref(
            uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)
        );
        bytes29 proofLeaves = txRootProofView.proofLeaves();
        uint txsLength = ckbTxProofArray.length;
        require(txsLength == ckbTxs.length, "length of ckbTxProofArray and ckbTxs mismatch");

        bytes29 txProofView;
        bytes29 rawTxView;
        bytes32 txHash;
        for (uint i = 0; i < txsLength; i++) {
            // - 1. check txHashes match from txProof and raw ckbTx
            txProofView = ckbTxProofArray[i].ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof));
//            txHash = txProofView.historyTxHash();
            rawTxView = txProofView.rawTransaction();

//
//            require(!usedTx_[txHash], "The burn tx cannot be reused");
//            usedTx_[txHash] = true;
//            require((txHash == CKBCrypto.digest(ckbTxs[i], ckbTxs[i].length)), "ckbTx mismatched with CkbHistoryTxProof");
//
//            // - 2. check txRoot match from txProof and txRootProof
//            uint leavesIndex = uint(txProofView.txRootProofLeavesIndex());
//            _proveTxExist(txProofView, proofLeaves.indexH256Array(leavesIndex));
//
//            // - 3. unlockToken
//            (uint256 bridgeAmount, uint256 bridgeFee, address tokenAddress, address recipientAddress) = decodeBurnResult(ckbTxs[i]);
//            require(bridgeAmount > bridgeFee, "fee should not exceed bridge amount");
//            uint256 receivedAmount = bridgeAmount - bridgeFee;
//            // address(0) means `ether` here
//            if (tokenAddress == address(0)) {
//                payable(recipientAddress).transfer(receivedAmount);
//                payable(msg.sender).transfer(bridgeFee);
//            } else {
//                IERC20(tokenAddress).safeTransfer(recipientAddress, receivedAmount);
//                IERC20(tokenAddress).safeTransfer(msg.sender, bridgeFee);
//            }
//            emit Unlocked(tokenAddress, recipientAddress, msg.sender, receivedAmount, bridgeFee);
        }
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

    function _proveTxExist(bytes29 txProofView, bytes32 txHash, bytes32 targetTxRoot)
    internal
    view
    returns (bool)
    {
        uint16 index = txProofView.txMerkleIndex();
        uint16 sibling;
        uint256 lemmasIndex = 0;
        bytes29 lemmas = txProofView.lemmas();
        uint256 length = lemmas.len() / 32;

        // calc the rawTransactionsRoot
        bytes32 rawTxRoot = txHash;
        while (lemmasIndex < length && index > 0) {
            sibling = ((index + 1) ^ 1) - 1;
            if (index < sibling) {
                rawTxRoot = Blake2b.digest64Merge(rawTxRoot, lemmas.indexH256Array(lemmasIndex));
            } else {
                rawTxRoot = Blake2b.digest64Merge(lemmas.indexH256Array(lemmasIndex), rawTxRoot);
            }

            lemmasIndex++;
            // index = parent(index)
            index = (index - 1) >> 1;
        }

        // calc the transactionsRoot by [rawTransactionsRoot, witnessesRoot]
        bytes32 transactionsRoot = Blake2b.digest64Merge(rawTxRoot, txProofView.witnessesRoot());
        require(
            transactionsRoot == targetTxRoot,
            "tx proof not passed"
        );
        return true;
    }
}
