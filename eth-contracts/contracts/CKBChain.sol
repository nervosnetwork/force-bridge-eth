pragma solidity ^0.5.10;

import {TypedMemView} from "./libraries/TypedMemView.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {ICKBChain} from "./interfaces/ICKBChain.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import {IHeaderVerifier} from "./interfaces/IHeaderVerifier.sol";

// tools below just for test, they will be removed before production ready
import "hardhat/console.sol";
import "./interfaces/IHeaderVerifier.sol";

contract CKBChain is ICKBChain, ICKBSpv {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using ViewSpv for bytes29;

    /// We store the hashes of the blocks for the past `CanonicalGcThreshold` headers.
    /// Events that happen past this threshold cannot be verified by the client.
    /// It is desirable that this number is larger than 7 days worth of headers, which is roughly
    /// 40k ckb blocks. So this number should be 40k in production.
    uint64 public CanonicalGcThreshold;
    /// We store full information about the headers for the past `FinalizedGcThreshold` blocks.
    /// This is required to be able to adjust the canonical chain when the fork switch happens.
    /// The commonly used number is 500 blocks, so this number should be 500 in production.
    uint64 public FinalizedGcThreshold;

    // Minimal information about the submitted block.
    struct BlockHeader {
        uint64 number;
        uint64 epoch;
        uint256 timestamp;
        uint256 totalDifficulty;
        bytes32 parentHash;
    }

    // Whether the contract was initialized.
    bool public initialized;
    uint64 public latestBlockNumber;
    BlockHeader latestHeader;

    // TODO remove `headerVerifierAddress`
    address public verifier;

    /// Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    /// entries.
    /// header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    /// TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    /// entries.
    /// header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionsRoots;


    /// All known header hashes. Stores up to `finalized_gc_threshold`.
    /// header number -> hashes of all headers with this number.
    mapping(uint64 => bytes32[]) allHeaderHashes;

    /// Known headers. Stores up to `finalized_gc_threshold`.
    mapping(bytes32 => BlockHeader) blockHeaders;

    /// @notice             requires `memView` to be of a specified type
    /// @param memView      a 29-byte view with a 5-byte type
    /// @param t            the expected type (e.g. BTCTypes.Outpoint, BTCTypes.TxIn, etc)
    /// @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, ViewCKB.CKBTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    /// #ICKBChain
    function addHeaders(bytes calldata data) external {
        // 1. view decode from data to headers view
        bytes29 headerVecView = data.ref(uint40(ViewCKB.CKBTypes.HeaderVec));

        // 2. iter headers
        uint32 length = headerVecView.lengthHeaderVec();
        uint32 index = 0;
        while (index < length) {
            bytes29 headerView = headerVecView.indexHeaderVec(index);
            _addHeader(headerView);
            index++;
        }
    }

    function _addHeader(bytes29 headerView) private {
        uint64 blockNumber = headerView.blockNumber();
        bytes32 blockHash = headerView.blockHash();
        uint256 mockTotalDifficulty = 0;

        // TODO verify version


        // verify blockNumber
        if (blockHeaders[headerView.parentHash()].number + 1 != blockNumber) {
            return;
        }

        // verify pow
        if (!IHeaderVerifier(verifier).verifyHeader(headerView.clone())) {
            return;
        }
        console.log('verify header success');


        // 1. insert to allHeaderHashes
        allHeaderHashes[blockNumber].push(blockHash);

        // 2. insert to blockHeaders
        blockHeaders[blockHash] = BlockHeader(
            blockNumber,
            headerView.epoch(),
            headerView.timestamp(),
            mockTotalDifficulty,
            headerView.parentHash()
        );

        // 3. refresh canonicalChain
        if (mockTotalDifficulty > latestHeader.totalDifficulty) {
            // remove lower difficulty canonical
            for (uint64 i = blockNumber + 1; i <= latestBlockNumber; i++) {
                delete canonicalTransactionsRoots[canonicalHeaderHashes[i]];
                delete canonicalHeaderHashes[i];
            }

            // set latest
            latestHeader = blockHeaders[blockHash];
            latestBlockNumber = blockNumber;

            // set canonical
            canonicalHeaderHashes[latestBlockNumber] = blockHash;
            canonicalTransactionsRoots[blockHash] = headerView.transactionsRoot();
            uint64 number = blockNumber - 1;
            bytes32 currentHash = latestHeader.parentHash;
            while (true) {
                if (number == 0 || canonicalHeaderHashes[number] == currentHash) {
                    break;
                }
                canonicalHeaderHashes[number] = currentHash;
                number--;
            }

            // gc
            if (blockNumber >= CanonicalGcThreshold) {
                _gcCanonicalChain(blockNumber - CanonicalGcThreshold);
            }

            if (blockNumber >= FinalizedGcThreshold) {
                _gcHeaders(blockNumber - FinalizedGcThreshold);
            }
        }
    }

    /// #ICKBSpv
    function proveTxExist(bytes calldata txProofData, uint64 numConfirmations) external view returns (bool) {
        bytes29 proofView = txProofData.ref(uint40(ViewSpv.SpvTypes.CKBTxProof));
        uint64 blockNumber = proofView.spvBlockNumber();
        bytes32 blockHash = proofView.blockHash();

        // TODO use safeMath for blockNumber + numConfirmations calc
        require(blockNumber + numConfirmations <= latestBlockNumber, "blockNumber from txProofData is too ahead of the latestBlockNumber");
        require(canonicalHeaderHashes[blockNumber] == blockHash, "blockNumber and blockHash mismatch");
        require(canonicalTransactionsRoots[blockHash] != bytes32(0), "blockHash invalid or too old");
        uint16 index = proofView.txMerkleIndex();
        uint16 sibling;
        uint256 lemmasIndex = 0;
        bytes29 lemmas = proofView.lemmas();
        uint256 length = lemmas.len() / 32;

        // calc the rawTransactionsRoot
        // TODO optimize rawTxRoot calculation with assembly code
        bytes32 rawTxRoot = proofView.txHash();
        while (lemmasIndex < length && index > 0) {
            sibling = ((index + 1) ^ 1) - 1;
            if (index < sibling) {
                rawTxRoot = CKBCrypto.digest(abi.encodePacked(rawTxRoot, lemmas.indexH256Array(lemmasIndex), new bytes(64)), 64);
            } else {
                rawTxRoot = CKBCrypto.digest(abi.encodePacked(lemmas.indexH256Array(lemmasIndex), rawTxRoot, new bytes(64)), 64);
            }

            lemmasIndex++;
            // index = parent(index)
            index = (index - 1) >> 1;
        }

        // calc the transactionsRoot by [rawTransactionsRoot, witnessesRoot]
        bytes32 transactionsRoot = CKBCrypto.digest(abi.encodePacked(rawTxRoot, proofView.witnessesRoot(), new bytes(64)), 64);
        require(transactionsRoot == canonicalTransactionsRoots[blockHash], "proof not passed");
        return true;
    }

    /// #GC
    /// Remove hashes from the Canonical chain that are at least as old as the given header number.
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

    /// Remove information about the headers that are at least as old as the given header number.
    function _gcHeaders(uint64 blockNumber) internal {
        uint64 number = blockNumber;
        while (true) {
            if (number == 0 || allHeaderHashes[number].length == 0) {
                break;
            }

            for (uint256 i = 0; i < allHeaderHashes[number].length; i++) {
                delete blockHeaders[allHeaderHashes[number][i]];
            }
            delete allHeaderHashes[number];
            number--;
        }
    }

    // TODO remove all mock function
    // mock for test
    function mockForProveTxExist(uint64 _latestBlockNumber, uint64 spvBlockNumber, bytes32 blockHash, bytes32 transactionsRoot) public {
        latestBlockNumber = _latestBlockNumber;
        canonicalHeaderHashes[spvBlockNumber] = blockHash;
        canonicalTransactionsRoots[blockHash] = transactionsRoot;
    }
    // mock for test verify
    function mockForHeaderVerifier(address _verifier) public {
        verifier = _verifier;
    }
}
