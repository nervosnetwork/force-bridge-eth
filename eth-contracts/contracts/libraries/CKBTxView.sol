// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./TypedMemView.sol";
import {SafeMath} from "./SafeMath.sol";

library CKBTxView {
    using TypedMemView for bytes29;
    using SafeMath for uint;

    enum CKBTxTypes {
        Unknown, // 0x0
        RawTx, // 0x1
        Outputs,
        OutputsData,
        RecipientCellOutput,
        RecipientCellData,
        Script,
        RecipientCellTypescriptArgs
    }

    modifier typeAssert(bytes29 memView, CKBTxTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    function outputs(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RawTx) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(20, 4);
        uint256 endIndex = _input.indexLEUint(24, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(CKBTxTypes.Outputs));
    }

    function outputsData(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RawTx) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(24, 4);
        uint256 endIndex = _input.indexLEUint(0, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(CKBTxTypes.OutputsData));
    }

    function recipientCellOutput(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Outputs) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex, len, uint40(CKBTxTypes.RecipientCellOutput));
    }

    function recipientCellData(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.OutputsData) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex + 4, len, uint40(CKBTxTypes.RecipientCellData));
    }

    function typescript(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellOutput) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex, len, uint40(CKBTxTypes.Script));
    }

    function recipientTypescriptCodeHash(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Script) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return _input.index(startIndex, 32);
    }

    function hashType(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Script) returns (uint8) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint8(_input.indexUint(startIndex, 1));
    }

    function recipientAddress(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (address) {
        return _input.indexAddress(0);
    }

    function tokenAddress(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (address) {
        return _input.indexAddress(20);
    }

    function contractAddress(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (address) {
        return _input.indexAddress(40);
    }

    function bridgeLockscriptCodeHash(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (bytes32) {
        return _input.index(60, 32);
    }

    function bridgeAmount(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (uint256) {
        return _input.indexLEUint(92, 16);
    }

    function bridgeFee(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (uint256) {
        return _input.indexLEUint(108, 16);
    }
}

