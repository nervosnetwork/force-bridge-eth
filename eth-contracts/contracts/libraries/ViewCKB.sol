// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./TypedMemView.sol";
import {SafeMath} from "./SafeMath.sol";

library ViewCKB {
    using TypedMemView for bytes29;
    using SafeMath for uint;
    uint256 public constant PERIOD_BLOCKS = 24 * 450 * 7;  // 1 week in blocks
    uint8 public constant NUMBER_SIZE = 4; // Size of Number in ckb molecule
    uint64 public constant NUMBER_MASK = 16777215;
    enum CKBTypes {
        Unknown, // 0x0
        Script, // 0x1
        Outpoint,
        CellInput,
        CellOutput,
        Bytes,
        H256,
        H160,
        Header,
        Nonce,
        RawHeader,
        Version,
        CompactTarget,
        Timestamp,
        BlockNumber,
        Epoch,
        ParentHash,
        TransactionsRoot,
        UnclesHash,

        HeaderVec,
        Transaction
    }

    // @notice             requires `memView` to be of a specified type
    // @param memView      a 29-byte view with a 5-byte type
    // @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    // @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, CKBTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    // @notice             extracts the since as an integer from a CellInput
    // @param _input       the CellInput
    // @return             the since
    function since(bytes29 _input) internal pure typeAssert(_input, CKBTypes.CellInput) returns (uint64) {
        return uint64(_input.indexLEUint(0, 8));
    }

    // @notice          extracts the outpoint from a CellInput
    // @param _input    the CellInput
    // @return          the outpoint as a typed memory
    function previousOutput(bytes29 _input) internal pure typeAssert(_input, CKBTypes.CellInput) returns (bytes29) {
        return _input.slice(8, 36, uint40(CKBTypes.Outpoint));
    }

    // @notice         extracts the codeHash from a Script
    // @param _input   the Script
    // @return         the codeHash
    function codeHash(bytes29 _input) internal pure typeAssert(_input, CKBTypes.Script) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return _input.index(startIndex, 32);
    }

    // @notice         extracts the hashType from a Script
    // @param _input   the Script
    // @return         the hashType
    function hashType(bytes29 _input) internal pure typeAssert(_input, CKBTypes.Script) returns (uint8) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint8(_input.indexUint(startIndex, 1));
    }

    // @notice         extracts the args from a Script
    // @param _input   the Script
    // @return         the args
    function args(bytes29 _input) internal pure typeAssert(_input, CKBTypes.Script) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(12, 4) + NUMBER_SIZE;
        uint256 inputLength = _input.len();
        return _input.slice(startIndex, inputLength - startIndex, uint40(CKBTypes.Bytes));
    }

    // @notice          extracts the rawHeader from a Header
    // @param _input    the Header
    // @return          the rawHeader as a typed memory
    function rawHeader(bytes29 _input) internal pure typeAssert(_input, CKBTypes.Header) returns (bytes29) {
        return _input.slice(0, 192, uint40(CKBTypes.RawHeader));
    }

    // @notice         extracts the nonce from a Header
    // @param _input   the Header
    // @return         the nonce
    function nonce(bytes29 _input) internal pure typeAssert(_input, CKBTypes.Header) returns (uint128) {
        return uint128(_input.indexLEUint(192, 16));
    }

    // @notice         extracts the version from a RawHeader
    // @param _input   the RawHeader
    // @return         the version
    function version(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (uint32) {
        return uint32(_input.indexLEUint(0, 4));
    }

    // @notice         extracts the compactTarget from a RawHeader
    // @param _input   the RawHeader
    // @return         the compactTarget
    function compactTarget(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (uint32) {
        return uint32(_input.indexLEUint(4, 4));
    }

    // @notice         extracts the timestamp from a RawHeader
    // @param _input   the RawHeader
    // @return         the timestamp
    function timestamp(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (uint64) {
        return uint64(_input.indexLEUint(8, 8));
    }

    // @notice         extracts the blockNumber from a RawHeader
    // @param _input   the RawHeader
    // @return         the blockNumber
    function blockNumber(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (uint64) {
        return uint64(_input.indexLEUint(16, 8));
    }

    // @notice         extracts the epoch from a RawHeader
    // @param _input   the RawHeader
    // @return         the epoch
    function epoch(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (uint64) {
        return uint64(_input.indexLEUint(24, 8)) & NUMBER_MASK;
    }

    // @notice         extracts the parentHash from a RawHeader
    // @param _input   the RawHeader
    // @return         the parentHash
    function parentHash(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (bytes32) {
        return _input.index(32, 32);
    }

    // @notice         extracts the transactionsRoot from a RawHeader
    // @param _input   the RawHeader
    // @return         the transactionsRoot
    function transactionsRoot(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (bytes32) {
        return _input.index(64, 32);
    }

    // @notice         extracts the unclesHash from a RawHeader
    // @param _input   the RawHeader
    // @return         the unclesHash
    function unclesHash(bytes29 _input) internal pure typeAssert(_input, CKBTypes.RawHeader) returns (bytes32) {
        return _input.index(128, 32);
    }

    // @notice         Index a header vector.
    // @dev            Errors on overruns
    // @param _headers The header vector
    // @param index    The 0-indexed location of the header to get
    // @return         the typed header at `index`
    function indexHeaderVec(bytes29 _headers, uint256 index) internal pure typeAssert(_headers, CKBTypes.HeaderVec) returns (bytes29) {
        uint256 _start = 4 + index.mul(208);
        return _headers.slice(_start, 208, uint40(CKBTypes.Header));
    }

    // @notice         return length of a header vector.
    // @dev            Errors on overruns
    // @param _headers The header vector
    // @return         the length of a header vector.
    function lengthHeaderVec(bytes29 _headers) internal pure typeAssert(_headers, CKBTypes.HeaderVec) returns (uint32) {
        return uint32(_headers.indexLEUint(0, 4));
    }
}
