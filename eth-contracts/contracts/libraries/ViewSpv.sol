// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./TypedMemView.sol";
import {ViewCKB} from "./ViewCKB.sol";
import {SafeMath} from "./SafeMath.sol";
import {CKBTxView} from "./CKBTxView.sol";

library ViewSpv {
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using SafeMath for uint;

    enum SpvTypes {
        Unknown,                // 0x0
        CKBTxProof,
        H256,
        H256Array,
        U64Array,
        CKBUnlockTokenParam,
        CKBHistoryTxRootProof,
        CKBHistoryTxProof,
        CKBHistoryTxProofVec
    }

    // @notice             requires `memView` to be of a specified type
    // @param memView      a 29-byte view with a 5-byte type
    // @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    // @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, SpvTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    // ## CkbTxProof
    function txMerkleIndex(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (uint16) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return uint16(_input.indexLEUint(startIndex, 2));
    }

    function spvBlockNumber(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (uint64) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint64(_input.indexLEUint(startIndex, 8));
    }

    function blockHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        return _input.index(startIndex, 32);
    }

    function txHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(16, 4);
        return _input.index(startIndex, 32);
    }

    function witnessesRoot(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(20, 4);
        return _input.index(startIndex, 32);
    }

    function lemmas(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(24, 4) + 4;
        uint256 inputLength = _input.len();
        return _input.slice(startIndex, inputLength - startIndex, uint40(SpvTypes.H256Array));
    }

    // @notice         Index a H256 array.
    // @dev            Errors on overruns
    // @param _arr     The H256 array
    // @param index    The 0-indexed location of the H256 to get
    // @return         the typed H256 at `index`
    function indexH256Array(bytes29 _arr, uint256 index) internal pure typeAssert(_arr, SpvTypes.H256Array) returns (bytes32) {
        uint256 _start = index.mul(32);
        return _arr.index(_start, 32);
    }

    function indexU64Array(bytes29 _arr, uint256 index) internal pure typeAssert(_arr, SpvTypes.U64Array) returns (uint64) {
        uint256 _start = index.mul(8);
        return uint64(_arr.indexLEUint(_start, 8));
    }

    // ## CkbHistoryTxProof
    function txBlockNumber(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProof) returns (uint64) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return uint64(_input.indexLEUint(startIndex, 8));
    }

    function historyTxMerkleIndex(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProof) returns (uint16) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint16(_input.indexLEUint(startIndex, 2));
    }

    function historyWitnessesRoot(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProof) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        return _input.index(startIndex, 32);
    }

    function historyLemmas(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProof) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(16, 4) + 4;
        uint256 endIndex = _input.indexLEUint(20, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(SpvTypes.H256Array));
    }

    function rawTransaction(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProof) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(20, 4) + 4;
        uint256 inputLength = _input.len();
        return _input.slice(startIndex, inputLength - startIndex, uint40(CKBTxView.CKBTxTypes.RawTx));
    }

    // ## CKBHistoryTxRootProof
    function initBlockNumber(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxRootProof) returns (uint64) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return uint64(_input.indexLEUint(startIndex, 8));
    }

    function latestBlockNumber(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxRootProof) returns (uint64) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint64(_input.indexLEUint(startIndex, 8));
    }

    function indices(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxRootProof) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(12, 4) + 4;
        uint256 endIndex = _input.indexLEUint(16, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(SpvTypes.U64Array));
    }

    function proofLeaves(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxRootProof) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(16, 4) + 4;
        uint256 endIndex = _input.indexLEUint(20, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(SpvTypes.H256Array));
    }

    function txRootLemmas(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxRootProof) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(20, 4) + 4;
        uint256 inputLength = _input.len();
        return _input.slice(startIndex, inputLength - startIndex, uint40(SpvTypes.H256Array));
    }

    // ## CKBUnlockTokenParam
    function historyTxRootProof(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBUnlockTokenParam) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 endIndex = _input.indexLEUint(8, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(SpvTypes.CKBHistoryTxRootProof));
    }

    function historyTxProofVec(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBUnlockTokenParam) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        uint256 inputLength = _input.len();
        return _input.slice(startIndex, inputLength - startIndex, uint40(SpvTypes.CKBHistoryTxProofVec));
    }

    function txProofLength(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProofVec) returns (uint) {
        if (_input.len() == 4) {
            return 0;
        }
        return _input.indexLEUint(4, 4) / 4 - 1;
    }

    function getHistoryTxProofFromVec(bytes29 _input, uint256 idx) internal pure typeAssert(_input, SpvTypes.CKBHistoryTxProofVec) returns (bytes29) {
        uint256 startIndex = 4 * (1 + idx);
        uint256 start = _input.indexLEUint(startIndex, 4);
        if (idx == txProofLength(_input) - 1) {
            uint256 inputLength = _input.len();
            return _input.slice(start, inputLength - start, uint40(SpvTypes.CKBHistoryTxProof));
        } else {
            uint256 endIndex = startIndex + 4;
            uint256 end = _input.indexLEUint(endIndex, 4);
            return _input.slice(start, end - start, uint40(SpvTypes.CKBHistoryTxProof));
        }
    }
}
