// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./libraries/TypedMemView.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {Blake2b} from "./libraries/Blake2b.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {CKBPow} from "./libraries/CKBPow.sol";
import {EaglesongLib} from "./libraries/EaglesongLib.sol";
import {ICKBChain} from "./interfaces/ICKBChain.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";

// tools below just for test, they will be removed before production ready
//import "hardhat/console.sol";

contract CKBChain is ICKBChain, ICKBSpv {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using ViewSpv for bytes29;

    // CHAIN_VERSION means chain_id on CKB CHAIN
    uint32 public constant CHAIN_VERSION = 0;

    // We store the hashes of the blocks for the past `CanonicalGcThreshold` headers.
    // Events that happen past this threshold cannot be verified by the client.
    // It is desirable that this number is larger than 7 days worth of headers, which is roughly
    // 40k ckb blocks. So this number should be 40k in production.
    uint64 public CanonicalGcThreshold;
    // We store full information about the headers for the past `FinalizedGcThreshold` blocks.
    // This is required to be able to adjust the canonical chain when the fork switch happens.
    // The commonly used number is 500 blocks, so this number should be 500 in production.
    uint64 public FinalizedGcThreshold;

    // Minimal information about the submitted block.
    struct BlockHeader {
        uint64 number;
        uint64 epoch;
        uint256 difficulty;
        uint256 totalDifficulty;
        bytes32 parentHash;
    }

    // Whether the contract was initialized.
    bool public initialized;
    uint64 public latestBlockNumber;
    uint64 public initBlockNumber;
    BlockHeader latestHeader;

    // Todo will remove the governance when optimistic phase
    address public governance;
    uint256 MOCK_DIFFICULTY = 1;

    // Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    // entries.
    // header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    // TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    // entries.
    // header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionsRoots;

    // All known header hashes. Stores up to `finalized_gc_threshold`.
    // header number -> hashes of all headers with this number.
    mapping(uint64 => bytes32[]) allHeaderHashes;

    // Known headers. Stores up to `finalized_gc_threshold`.
    mapping(bytes32 => BlockHeader) blockHeaders;

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

    constructor() {
        governance = msg.sender;
    }

    // query
    function getLatestBlockNumber() public view returns (uint64) {
        return latestBlockNumber;
    }

    // query
    function getHeadersByNumber(uint64 blockNumber)
        public
        view
        returns (bytes32[] memory)
    {
        return allHeaderHashes[blockNumber];
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

    // query
    function getLatestEpoch() public view returns (uint64) {
        return latestHeader.epoch;
    }

    function initWithHeader(
        bytes calldata data,
        bytes32 blockHash,
        uint64 finalizedGcThreshold,
        uint64 canonicalGcThreshold
    ) external onlyGov {
        require(!initialized, "Contract is already initialized");
        initialized = true;

        // set init threshold
        FinalizedGcThreshold = finalizedGcThreshold;
        CanonicalGcThreshold = canonicalGcThreshold;

        // decoder init header
        bytes29 rawHeaderView = data
            .ref(uint40(ViewCKB.CKBTypes.Header))
            .rawHeader();
        (uint256 target, ) = CKBPow.compactToTarget(
            rawHeaderView.compactTarget()
        );
        uint256 difficulty = CKBPow.targetToDifficulty(target);
        uint64 blockNumber = rawHeaderView.blockNumber();
        initBlockNumber = blockNumber;
        BlockHeader memory header = BlockHeader(
            blockNumber,
            rawHeaderView.epoch(),
            difficulty,
            difficulty,
            rawHeaderView.parentHash()
        );

        // set headers
        allHeaderHashes[blockNumber].push(blockHash);
        blockHeaders[blockHash] = header;

        // set canonical chain
        _refreshCanonicalChain(header, blockHash);
        canonicalTransactionsRoots[blockHash] = rawHeaderView
            .transactionsRoot();
    }

    // # ICKBChain
    // TODO add onlyGov for phase1 production
    function addHeaders(bytes calldata data) override external {
        require(initialized, "Contract is not initialized");

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
        bytes29 rawHeaderView = headerView.rawHeader();
        uint64 blockNumber = rawHeaderView.blockNumber();

        // calc blockHash
        bytes32 blockHash = Blake2b.digest208Ptr(headerView.loc());

        // ## verify blockHash should not exist
        if (
            canonicalTransactionsRoots[blockHash] != bytes32(0) ||
            blockHeaders[blockHash].number == blockNumber
        ) {
            return;
        }

        // ## verify blockNumber
        require(
            blockNumber + CanonicalGcThreshold >= latestBlockNumber,
            "block is too old"
        );
        bytes32 parentHash = rawHeaderView.parentHash();

        BlockHeader memory parentHeader = blockHeaders[parentHash];
        require(
            parentHeader.totalDifficulty > 0 &&
                parentHeader.number + 1 == blockNumber,
            "block's parent block mismatch"
        );

        // ## verify pow
        // uint256 difficulty = _verifyPow(headerView, rawHeaderView, parentHeader);
        uint256 difficulty = MOCK_DIFFICULTY;

        // ## insert header to storage
        // 1. insert to blockHeaders
        BlockHeader memory header = BlockHeader(
            blockNumber,
            rawHeaderView.epoch(),
            difficulty,
            parentHeader.totalDifficulty + difficulty,
            parentHash
        );
        blockHeaders[blockHash] = header;

        // 2. insert to allHeaderHashes
        allHeaderHashes[header.number].push(blockHash);

        // 3. refresh canonicalChain
        if (header.totalDifficulty > latestHeader.totalDifficulty) {
            _refreshCanonicalChain(header, blockHash);
            canonicalTransactionsRoots[blockHash] = rawHeaderView
                .transactionsRoot();
        }
    }

    // @notice                     verifyPow for the header
    // @dev                        reference code:  https://github.com/nervosnetwork/ckb/blob/develop/pow/src/eaglesong.rs
    // @param headerView           the bytes29 view of the header
    // @param parentHeader         parent header of the header
    // @return                     the difficulty of the header
    function _verifyPow(
        bytes29 headerView,
        bytes29 rawHeaderView,
        BlockHeader memory parentHeader
    ) internal view returns (uint256) {
        bytes32 rawHeaderHash = CKBCrypto.digest(rawHeaderView.clone(), 192);

        // - 1. calc powMessage
        bytes memory powMessage = abi.encodePacked(
            rawHeaderHash,
            headerView.slice(192, 16, uint40(ViewCKB.CKBTypes.Nonce)).clone()
        );

        // - 2. calc EaglesongHash to output
        bytes32 output = EaglesongLib.EaglesongHash(powMessage);

        // - 3. calc block_target, check if target > 0
        (uint256 target, bool overflow) = CKBPow.compactToTarget(
            rawHeaderView.compactTarget()
        );
        require(target > 0 && !overflow, "block target is zero or overflow");

        // - 4. check if EaglesongHash <= block target
        // @dev the smaller the target value, the greater the difficulty
        require(
            uint256(output) <= target,
            "block difficulty should greater or equal the target difficulty"
        );

        // - 5. verify_difficulty
        uint256 difficulty = CKBPow.targetToDifficulty(target);
        uint64 epoch = rawHeaderView.epoch();
        if (epoch == parentHeader.epoch) {
            require(
                difficulty == parentHeader.difficulty,
                "difficulty should equal parent's difficulty"
            );
        } else {
            // we are using dampening factor τ = 2 in CKB, the difficulty adjust range will be [previous / (τ * τ) .. previous * (τ * τ)]
            require(
                difficulty >= parentHeader.difficulty / 4 &&
                    difficulty <= parentHeader.difficulty * 4,
                "difficulty invalid"
            );
        }

        return difficulty;
    }

    function _refreshCanonicalChain(
        BlockHeader memory header,
        bytes32 blockHash
    ) internal {
        // remove lower difficulty canonical branch
        for (uint64 i = header.number + 1; i <= latestBlockNumber; i++) {
            emit BlockHashReverted(i, canonicalHeaderHashes[i]);
            delete canonicalTransactionsRoots[canonicalHeaderHashes[i]];
            delete canonicalHeaderHashes[i];
        }

        // set latest
        latestHeader = header;
        latestBlockNumber = header.number;

        // set canonical
        canonicalHeaderHashes[latestBlockNumber] = blockHash;
        emit BlockHashAdded(latestBlockNumber, blockHash);

        // set new branch to canonical chain
        uint64 parentNumber = latestBlockNumber - 1;
        bytes32 parentHash = latestHeader.parentHash;
        while (parentNumber > 0) {
            if (
                canonicalHeaderHashes[parentNumber] == bytes32(0) ||
                canonicalHeaderHashes[parentNumber] == parentHash
            ) {
                break;
            }
            canonicalHeaderHashes[parentNumber] = parentHash;

            parentHash = blockHeaders[parentHash].parentHash;
            parentNumber--;
        }

        // gc
        if (header.number >= CanonicalGcThreshold) {
            _gcCanonicalChain(header.number - CanonicalGcThreshold);
        }

        if (header.number >= FinalizedGcThreshold) {
            _gcHeaders(header.number - FinalizedGcThreshold);
        }
    }

    // #ICKBSpv
    function proveTxExist(bytes calldata txProofData, uint64 numConfirmations)
        override
        external
        view
        returns (bool)
    {
        require(initialized, "Contract is not initialized");

        bytes29 proofView = txProofData.ref(
            uint40(ViewSpv.SpvTypes.CKBTxProof)
        );
        uint64 blockNumber = proofView.spvBlockNumber();
        bytes32 blockHash = proofView.blockHash();

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

    // Remove information about the headers that are at least as old as the given header number.
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

    // TODO remove all mock function before production ready
    // mock for test
    function mockForProveTxExist(
        uint64 _latestBlockNumber,
        uint64 spvBlockNumber,
        bytes32 blockHash,
        bytes32 transactionsRoot
    ) public {
        governance = msg.sender;
        initialized = true;
        latestBlockNumber = _latestBlockNumber;
        canonicalHeaderHashes[spvBlockNumber] = blockHash;
        canonicalTransactionsRoots[blockHash] = transactionsRoot;
    }
}
