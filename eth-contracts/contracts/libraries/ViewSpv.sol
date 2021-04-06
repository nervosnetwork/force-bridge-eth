// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./TypedMemView.sol";
import {SafeMath} from "./SafeMath.sol";

library ViewSpv {
    using TypedMemView for bytes29;
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
        CKBHistoryTxProofVec, 

        // ckb tx
        RawTx,
        Outputs,
        OutputsData,
        RecipientCellOutput,
        RecipientCellData,
        Script,
        RecipientCellTypescriptArgs
    }

    // @notice             requires `memView` to be of a specified type
    // @param memView      a 29-byte view with a 5-byte type
    // @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    // @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, SpvTypes t) {
        memView.assertType(uint40(t));
        _;
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
        return _input.slice(startIndex, inputLength - startIndex, uint40(SpvTypes.RawTx));
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
    
    // ckb tx view
    function outputs(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RawTx) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(20, 4);
        uint256 endIndex = _input.indexLEUint(24, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(SpvTypes.Outputs));
    }

    function outputsData(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RawTx) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(24, 4);
        uint256 endIndex = _input.indexLEUint(0, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(SpvTypes.OutputsData));
    }

    function recipientCellOutput(bytes29 _input) internal pure typeAssert(_input, SpvTypes.Outputs) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex, len, uint40(SpvTypes.RecipientCellOutput));
    }

    function recipientCellData(bytes29 _input) internal pure typeAssert(_input, SpvTypes.OutputsData) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex + 4, len, uint40(SpvTypes.RecipientCellData));
    }

    function typescript(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellOutput) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex, len, uint40(SpvTypes.Script));
    }

    function recipientTypescriptCodeHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.Script) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return _input.index(startIndex, 32);
    }

    function hashType(bytes29 _input) internal pure typeAssert(_input, SpvTypes.Script) returns (uint8) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint8(_input.indexUint(startIndex, 1));
    }

    function recipientAddress(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (address) {
        return _input.indexAddress(0);
    }

    function tokenAddress(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (address) {
        return _input.indexAddress(20);
    }

    function contractAddress(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (address) {
        return _input.indexAddress(40);
    }

    function lightClientTypescriptHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (bytes32) {
        return _input.index(60, 32);
    }

    function bridgeLockscriptCodeHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (bytes32) {
        return _input.index(92, 32);
    }

    function bridgeAmount(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (uint256) {
        return _input.indexLEUint(124, 16);
    }

    function bridgeFee(bytes29 _input) internal pure typeAssert(_input, SpvTypes.RecipientCellData) returns (uint256) {
        return _input.indexLEUint(140, 16);
    }
}



