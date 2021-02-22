// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {SafeERC20} from "./libraries/SafeERC20.sol";
import {IERC20} from "./interfaces/IERC20.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import {Blake2b} from "./libraries/Blake2b.sol";

//import "hardhat/console.sol";

contract TokenLocker {
    using Address for address;
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
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

    struct TreeNode {
        uint64 index;
        bytes32 data;
    }

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

    function unlockToken(bytes memory proof) external {
        // 1. getHistoryTxRootInfo from CkbChain
        (uint64 initBlockNumber, uint64 latestBlockNumber, bytes32 targetHistoryTxRoot) = ckbSpv_.getHistoryTxRootInfo();
        require(latestBlockNumber > 0, "ckbSpv should initialize");

        // 2. check proveTxRootExist
        bytes29 txProofVecView;
        TreeNode[] memory leafNodes;
        {
            bytes29 proofView = proof.ref(uint40(ViewSpv.SpvTypes.CKBUnlockTokenParam));
            bytes29 txRootProofView = proofView.historyTxRootProof();
            txProofVecView = proofView.historyTxProofVec();

            require(txRootProofView.initBlockNumber() == initBlockNumber, "initBlockNumber mismatch");
            require(txRootProofView.latestBlockNumber() == latestBlockNumber, "latestBlockNumber mismatch");
            leafNodes = _proveTxRootExist(txRootProofView, targetHistoryTxRoot);
        }

        // 3. check txs exist and unlockToken
        bytes29 txProofView;
        uint64 blockNumber;
        uint length = txProofVecView.txProofLength();
        uint64 merkleIndex;
        for (uint i = 0; i < length; i++) {
            txProofView = txProofVecView.getHistoryTxProofFromVec(i);
            blockNumber = txProofView.txBlockNumber();
            require(
                blockNumber >= initBlockNumber,
                "the blockNumber which the tx exists in should be greater than or equal to initBlockNumber"
            );
            require(
                uint256(blockNumber) + uint256(numConfirmations_) <= uint256(latestBlockNumber),
                "blockNumber from txProofData should not be greater than latestBlockNumber"
            );

            // - 1. check if txHashes from txProof and raw ckbTx match
            bytes memory rawTx = txProofView.rawTransaction().clone();
            bytes32 txHash = CKBCrypto.digest(rawTx, rawTx.length);
            require(!usedTx_[txHash], "The burn tx cannot be reused");
            usedTx_[txHash] = true;

            // - 2. proveTxExist, check if txRoots from txProof and txRootProof match
            // calc the index in txRoot-merkle-tree
            // @dev refer to https://github.com/nervosnetwork/merkle-tree/blob/master/README.md
            // in definition, merkleIndex = i+n-1
            // n == all_leaves_count == latestBlockNumber - initBlockNumber + 1
            // i == index of the item in all_leaves == blockNumber - initBlockNumber
            // merkleIndex = ( blockNumber - initBlockNumber ) + (latestBlockNumber - initBlockNumber + 1) - 1
            merkleIndex = (latestBlockNumber - initBlockNumber) + (blockNumber - initBlockNumber);
            _proveTxExist(txProofView, txHash, _getTargetTxRoot(merkleIndex, leafNodes));

            // - 3. unlockToken
            (uint256 bridgeAmount, uint256 bridgeFee, address tokenAddress, address recipientAddress) = decodeBurnResult(rawTx);
            require(bridgeAmount > bridgeFee, "fee should not exceed bridge amount");
            uint256 receivedAmount = bridgeAmount - bridgeFee;
            // address(0) means `ether` here
            if (tokenAddress == address(0)) {
                payable(recipientAddress).transfer(receivedAmount);
                payable(msg.sender).transfer(bridgeFee);
            } else {
                IERC20(tokenAddress).safeTransfer(recipientAddress, receivedAmount);
                IERC20(tokenAddress).safeTransfer(msg.sender, bridgeFee);
            }
            emit Unlocked(tokenAddress, recipientAddress, msg.sender, receivedAmount, bridgeFee);
        }
    }

    function decodeBurnResult(bytes memory ckbTx) public view returns (
        uint256 bridgeAmount,
        uint256 bridgeFee,
        address token,
        address recipient
    ){
        bytes29 rawTx = ckbTx.ref(uint40(ViewSpv.SpvTypes.RawTx));
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

    function _getTargetTxRoot(uint64 leafMerkleIndex, TreeNode[] memory leafNodes) internal pure returns (bytes32) {
        for (uint i = 0; i < leafNodes.length; i++) {
            if (leafMerkleIndex == leafNodes[i].index) {
                return leafNodes[i].data;
            }
        }
        return bytes32(0);
    }

    function _proveTxExist(bytes29 txProofView, bytes32 txHash, bytes32 targetTxRoot)
    internal
    view
    returns (bool)
    {
        require(targetTxRoot != bytes32(0), "txRoot from the blockNumber is not in the proof");
        uint16 index = txProofView.historyTxMerkleIndex();
        uint16 sibling;
        uint256 lemmasIndex = 0;
        bytes29 lemmas = txProofView.historyLemmas();
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
        bytes32 transactionsRoot = Blake2b.digest64Merge(rawTxRoot, txProofView.historyWitnessesRoot());
        require(
            transactionsRoot == targetTxRoot,
            "tx proof not passed"
        );
        return true;
    }

    function _proveTxRootExist(bytes29 txRootProofView, bytes32 targetHistoryTxRoot)
    internal
    pure
    returns (TreeNode[] memory leafNodes)
    {
        // queue
        bytes29 indices = txRootProofView.indices();
        uint leavesLength = indices.len() / 8;
        uint queueLength = leavesLength + 1;
        TreeNode[] memory queue = new TreeNode[](queueLength);
        leafNodes = new TreeNode[](leavesLength);
        uint front = 0;
        uint end = 0;

        // merkle tree indices and node(byte32) of leaves
        {
            bytes29 proofLeaves = txRootProofView.proofLeaves();
            require(leavesLength > 0 && leavesLength == proofLeaves.len() / 32, "length of indices and proofLeaves mismatch");
            TreeNode memory node;
            for (uint i = 0; i < leavesLength; i++) {
                node = TreeNode(indices.indexU64Array(i), proofLeaves.indexH256Array(i));
                leafNodes[i] = node;
                queue[end] = node;
                end++;
            }
        }

        //  merkle tree lemmas
        uint lemmasPosition = 0;
        bytes29 lemmas = txRootProofView.txRootLemmas();
        uint lemmasLength = lemmas.len() / 32;

        // init
        uint64 currentIndex;
        bytes32 currentNode;
        uint64 siblingIndex;
        bytes32 siblingNode;

        while (front != end) {
            currentIndex = queue[front].index;
            currentNode = queue[front].data;
            front = (front + 1) % queueLength;

            if (currentIndex == 0) {
                break;
            }

            siblingIndex = ((currentIndex + 1) ^ 1) - 1;
            if (front != end && siblingIndex == queue[front].index) {
                siblingNode = queue[front].data;
                front = (front + 1) % queueLength;
            } else {
                require(lemmasPosition < lemmasLength, "invalid historyTxRootProof");
                siblingNode = lemmas.indexH256Array(lemmasPosition);
                lemmasPosition++;
            }

            // push parentTreeNode to queue
            // parentIndex == (currentIndex - 1) >> 1, parentNode
            if (currentIndex < siblingIndex) {
                queue[end] = TreeNode((currentIndex - 1) >> 1, keccak256(abi.encodePacked(currentNode, siblingNode)));
            } else {
                queue[end] = TreeNode((currentIndex - 1) >> 1, keccak256(abi.encodePacked(siblingNode, currentNode)));
            }
            end = (end + 1) % queueLength;
        }

        require(
            currentNode == targetHistoryTxRoot,
            "proof not verified"
        );
        return leafNodes;
    }
}
