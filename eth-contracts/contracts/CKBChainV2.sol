// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./libraries/TypedMemView.sol";
import {Blake2b} from "./libraries/Blake2b.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {ICKBChainV2} from "./interfaces/ICKBChainV2.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";

// tools below just for test, they will be removed before production ready
//import "hardhat/console.sol";

contract CKBChainV2 is ICKBChainV2, ICKBSpv {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using ViewSpv for bytes29;

    // We store the hashes of the blocks for the past `CanonicalGcThreshold` headers.
    // Events that happen past this threshold cannot be verified by the client.
    // It is desirable that this number is larger than 7 days worth of headers, which is roughly
    // 40k ckb blocks. So this number should be 40k in production.
    uint64 public CanonicalGcThreshold;

    uint64 public latestBlockNumber;
    uint64 public initBlockNumber;

    // Todo will remove the governance when optimistic phase
    address public governance;

    // Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    // entries.
    // header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    // TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    // entries.
    // header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionsRoots;

    // @notice             requires `memView` to be of a specified type
    // @param memView      a 29-byte view with a 5-byte type
    // @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    // @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, ViewCKB.CKBTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    /**
     * @dev Throws if called by any account other than the governance.
     */
    modifier onlyGov() {
        require(msg.sender == governance, "caller is not the governance");
        _;
    }

    constructor(uint64 canonicalGcThreshold) {
        governance = msg.sender;

        // set init threshold
        CanonicalGcThreshold = canonicalGcThreshold;
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
    function addHeaders(bytes[] calldata tinyHeaders) override external onlyGov {
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

    // #ICKBSpv
    function proveTxExist(bytes calldata txProofData, uint64 numConfirmations)
        override
        external
        view
        returns (bool)
    {
        bytes29 proofView = txProofData.ref(
            uint40(ViewSpv.SpvTypes.CKBTxProof)
        );
        uint64 blockNumber = proofView.spvBlockNumber();
        bytes32 blockHash = proofView.blockHash();

        // TODO use safeMath for blockNumber + numConfirmations calc
        require(
            blockNumber + numConfirmations <= latestBlockNumber,
            "blockNumber from txProofData is too ahead of the latestBlockNumber"
        );
        require(
            canonicalHeaderHashes[blockNumber] == blockHash,
            "blockNumber and blockHash mismatch"
        );
        require(
            canonicalTransactionsRoots[blockHash] != bytes32(0),
            "blockHash invalid or too old"
        );
        uint16 index = proofView.txMerkleIndex();
        uint16 sibling;
        uint256 lemmasIndex = 0;
        bytes29 lemmas = proofView.lemmas();
        uint256 length = lemmas.len() / 32;

        // calc the rawTransactionsRoot
        bytes32 rawTxRoot = proofView.txHash();
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
        bytes32 transactionsRoot = Blake2b.digest64Merge(rawTxRoot, proofView.witnessesRoot());
        require(
            transactionsRoot == canonicalTransactionsRoots[blockHash],
            "proof not passed"
        );
        return true;
    }
}
