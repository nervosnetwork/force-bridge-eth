// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./libraries/TypedMemView.sol";
import {Blake2b} from "./libraries/Blake2b.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {ICKBChainV2} from "./interfaces/ICKBChainV2.sol";
import {ICKBChainV3} from "./interfaces/ICKBChainV3.sol";
import {ICKBSpvV3} from "./interfaces/ICKBSpvV3.sol";
import {MultisigUtils} from "./libraries/MultisigUtils.sol";

// tools below just for test, they will be removed before production ready
//import "./test/console.sol";

contract CKBChainV3 is ICKBChainV2, ICKBChainV3, ICKBSpvV3 {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using ViewSpv for bytes29;

    bool public initialized;
    // We store the hashes of the blocks for the past `CanonicalGcThreshold` headers.
    // Events that happen past this threshold cannot be verified by the client.
    // It is desirable that this number is larger than 7 days worth of headers, which is roughly
    // 40k ckb blocks. So this number should be 40k in production.
    uint64 public CanonicalGcThreshold;

    uint64 public latestBlockNumber;
    uint64 public initBlockNumber;

    address public governance;

    // Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    // entries.
    // header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    // TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    // entries.
    // header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionsRoots;

    // refer to https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md
    uint public constant SIGNATURE_SIZE = 65;
    uint public constant VALIDATORS_SIZE_LIMIT = 20;
    string public constant NAME_712 = "Force Bridge CKBChain";
    // ADD_HEADERS_TYPEHASH = keccak256("AddHeaders(bytes[] tinyHeaders)");
    bytes32 public constant ADD_HEADERS_TYPEHASH = 0x1dac851def8ec317cf44b4a6cf63dabe82895259e6290d4c2ef271700bfce584;
    bytes32 public DOMAIN_SEPARATOR;
    // if the number of verified signatures has reached `multisigThreshold_`, validators approve the tx
    uint public multisigThreshold_;
    address[] validators_;

    // CKBChainV3-----------------------------
    // TODO modify ADD_HISTORY_TX_ROOT_TYPEHASH to correct value
    // ADD_HISTORY_TX_ROOT_TYPEHASH = keccak256("AddHistoryTxRoot(uint64 startBlockNumber, uint64 endBlockNumber, bytes32 historyTxRoot)");
    bytes32 public constant ADD_HISTORY_TX_ROOT_TYPEHASH = 0x0eeee1be1069b2c737b19f6c3510ceeed099af9ee1f5985109f117ce0524ca10;
    bytes32 public historyTxRoot;
    mapping(bytes32 => bytes32) verifiedTxRoots;
    struct TreeNode {
        uint16 index;
        bytes32 data;
    }

    // @notice             requires `memView` to be of a specified type
    // @param memView      a 29-byte view with a 5-byte type
    // @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    // @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, ViewCKB.CKBTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    /**
     * @notice  if addr is not one of validators_, return validators_.length
     * @return  index of addr in validators_
     */
    function getIndexOfValidators(address user) internal view returns (uint) {
        for (uint i = 0; i < validators_.length; i++) {
            if (validators_[i] == user) {
                return i;
            }
        }
        return validators_.length;
    }


    /**
     * @notice             @dev signatures are a multiple of 65 bytes and are densely packed.
     * @param signatures   The signatures bytes array
     */
    function validatorsApprove(bytes32 msgHash, bytes memory signatures, uint threshold) public view {
        require(signatures.length % SIGNATURE_SIZE == 0, "invalid signatures");

        // 1. check length of signature
        uint length = signatures.length / SIGNATURE_SIZE;
        require(length >= threshold, "length of signatures must greater than threshold");

        // 3. check number of verified signatures >= threshold
        uint verifiedNum = 0;
        uint i = 0;

        uint8 v;
        bytes32 r;
        bytes32 s;
        address recoveredAddress;
        // set indexVisited[ index of recoveredAddress in validators_ ] = true
        bool[] memory validatorIndexVisited = new bool[](validators_.length);
        uint validatorIndex;
        while (i < length) {
            (v, r, s) = MultisigUtils.parseSignature(signatures, i);
            i++;

            recoveredAddress = ecrecover(msgHash, v, r, s);
            require(recoveredAddress != address(0), "invalid signature");

            // get index of recoveredAddress in validators_
            validatorIndex = getIndexOfValidators(recoveredAddress);

            // recoveredAddress is not validator or has been visited
            if (validatorIndex >= validators_.length || validatorIndexVisited[validatorIndex]) {
                continue;
            }

            // recoveredAddress verified
            validatorIndexVisited[validatorIndex] = true;
            verifiedNum++;
            if (verifiedNum >= threshold) {
                return;
            }
        }

        require(verifiedNum >= threshold, "signatures not verified");
    }

    function initialize(
        uint64 canonicalGcThreshold,
        address[] memory validators,
        uint multisigThreshold
    ) public {
        require(!initialized, "Contract instance has already been initialized");
        initialized = true;

        // set init threshold
        CanonicalGcThreshold = canonicalGcThreshold;

        // set DOMAIN_SEPARATOR
        uint chainId;
        assembly {
            chainId := chainid()
        }
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(NAME_712)),
                keccak256(bytes("1")),
                chainId,
                address(this)
            )
        );

        // set validators
        require(validators.length <= VALIDATORS_SIZE_LIMIT, "number of validators exceeds the limit");
        validators_ = validators;
        require(multisigThreshold <= validators.length, "invalid multisigThreshold");
        multisigThreshold_ = multisigThreshold;
    }

    // query
    function getLatestBlockNumber() view public returns (uint64) {
        return latestBlockNumber;
    }

    // query
    function getCanonicalHeaderHash(uint64 blockNumber)
        public
        view
        returns (bytes32)
    {
        return canonicalHeaderHashes[blockNumber];
    }

    // query
    function getCanonicalTransactionsRoot(bytes32 blockHash)
        public
        view
        returns (bytes32)
    {
        return canonicalTransactionsRoots[blockHash];
    }

    // # ICKBChain
    function addHeaders(bytes[] calldata tinyHeaders, bytes calldata signatures) override external {
        // 1. calc msgHash
        bytes32 msgHash = keccak256(
            abi.encodePacked(
                "\x19\x01", // solium-disable-line
                DOMAIN_SEPARATOR,
                keccak256(abi.encode(ADD_HEADERS_TYPEHASH, tinyHeaders))
            )
        );

        // 2. validatorsApprove
        validatorsApprove(msgHash, signatures, multisigThreshold_);

        // 3. addHeaders
        bytes29 tinyHeaderView;
        for (uint i = 0; i < tinyHeaders.length; i++) {
            tinyHeaderView = tinyHeaders[i].ref(uint40(ViewCKB.CKBTypes.TinyHeader));
            _addHeader(tinyHeaderView);
        }
    }

    function _addHeader(bytes29 tinyHeaderView) private {
        bytes32 blockHash = tinyHeaderView.hash();

        // 1. set latestBlockNumber
        latestBlockNumber = tinyHeaderView.number();

        // 1. refresh canonicalChain
        canonicalHeaderHashes[latestBlockNumber] = blockHash;
        canonicalTransactionsRoots[blockHash] = tinyHeaderView.txRoot();
        emit BlockHashAdded(latestBlockNumber, blockHash);

        // 2. gc
        if (latestBlockNumber > CanonicalGcThreshold) {
            _gcCanonicalChain(latestBlockNumber - CanonicalGcThreshold);
        }
    }

    // Remove hashes from the Canonical chain that are at least as old as the given header number.
    function _gcCanonicalChain(uint64 blockNumber) internal {
        uint64 number = blockNumber;
        while (true) {
            if (number == 0 || canonicalHeaderHashes[number] == bytes32(0)) {
                break;
            }

            delete canonicalTransactionsRoots[canonicalHeaderHashes[number]];
            delete canonicalHeaderHashes[number];
            number--;
        }
    }

    // CKBChainV3-----------------------------
    function addHistoryTxRoot(uint64 _initBlockNumber, uint64 _latestBlockNumber, bytes32 _historyTxRoot, bytes calldata signatures)
    override
    external
    {
        // 1. calc msgHash
        bytes32 msgHash = keccak256(
            abi.encodePacked(
                "\x19\x01", // solium-disable-line
                DOMAIN_SEPARATOR,
                keccak256(abi.encode(ADD_HISTORY_TX_ROOT_TYPEHASH, _initBlockNumber, _latestBlockNumber, _historyTxRoot))
            )
        );

        // 2. validatorsApprove
        validatorsApprove(msgHash, signatures, multisigThreshold_);

        initBlockNumber = _initBlockNumber;
        latestBlockNumber = _latestBlockNumber;
        historyTxRoot = _historyTxRoot;

        // 3. event
        emit HistoryTxRootAdded(_initBlockNumber, _latestBlockNumber, _historyTxRoot);
    }

    function proveTxRootExist(bytes calldata txRootProofData)
    override
    external
    view
    returns (bool)
    {
        bytes29 txRootProofView = txRootProofData.ref(
            uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)
        );

        // queue
        bytes29 indices = txRootProofView.indices();
        uint leavesLength = indices.len() / 2;
        uint queueLength = leavesLength + 1;
        TreeNode[] memory queue = new TreeNode[](queueLength);
        uint front = 0;
        uint end = 0;

        // merkle tree indices and node(byte32) of leaves
        {
            bytes29 proofLeaves = txRootProofView.proofLeaves();
            require(leavesLength > 0 && leavesLength == proofLeaves.len() / 32, "length of indices and proofLeaves mismatch");
            for (uint i = 0; i < leavesLength; i++) {
                queue[end] = TreeNode(indices.indexU16Array(i), proofLeaves.indexH256Array(i));
                end++;
            }
        }

        //  merkle tree lemmas
        uint lemmasPosition = 0;
        bytes29 lemmas = txRootProofView.txRootLemmas();
        uint lemmasLength = lemmas.len() / 32;

        // init
        uint16 currentIndex;
        bytes32 currentNode;
        uint16 siblingIndex;
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
            // TODO modify Blake2b.digest64Merge to keccak256
            if (currentIndex < siblingIndex) {
                queue[end] = TreeNode((currentIndex - 1) >> 1, Blake2b.digest64Merge(currentNode, siblingNode));
            } else {
                queue[end] = TreeNode((currentIndex - 1) >> 1, Blake2b.digest64Merge(siblingNode, currentNode));
            }
            end = (end + 1) % queueLength;
        }

        require(
            currentNode == historyTxRoot,
            "proof not verified"
        );
        return true;
    }
}
