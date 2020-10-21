pragma solidity ^0.5.10;

import {TypedMemView} from "./libraries/TypedMemView.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {ICKBChain} from "./interfaces/ICKBChain.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";

contract CKBChain is ICKBChain, ICKBSpv {
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
    uint64 public lastBlockNumber;

    /// Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    /// entries.
    /// header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    /// TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    /// entries.
    /// header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionRoots;


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
    }

    /// #ICKBSpv
    function proveTxExist(bytes calldata txProofData, uint64 numConfirmations) external view returns (bool){
        return true;
    }

    /// #GC
    /// Remove hashes from the active chain that are at least as old as the given header number.
    function gcActiveChain(uint64 blockNumber) internal {
    }

    /// Remove information about the headers that are at least as old as the given header number.
    function gcHeaders(uint64 blockNumber) internal {
    }

    /// #Verify header
    function verifyHeader(bytes29 header) internal view typeAssert(header, ViewCKB.CKBTypes.Header) returns (bool){
        return true;
    }
}
