pragma solidity ^0.5.10;

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

    /// @notice             requires `memView` to be of a specified type
    /// @param memView      a 29-byte view with a 5-byte type
    /// @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    /// @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, CKBTxTypes t) {
        memView.assertType(uint40(t));
        _;
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function outputs(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RawTx) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(20, 4);
        uint256 endIndex = _input.indexLEUint(24, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(CKBTxTypes.Outputs));
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function outputsData(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RawTx) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(24, 4);
        uint256 endIndex = _input.indexLEUint(0, 4);
        return _input.slice(startIndex, endIndex - startIndex, uint40(CKBTxTypes.OutputsData));
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function recipientCellOutput(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Outputs) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex, len, uint40(CKBTxTypes.RecipientCellOutput));
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function recipientCellData(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.OutputsData) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex + 4, len, uint40(CKBTxTypes.RecipientCellData));
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function typescript(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellOutput) returns (bytes29) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        return _input.slice(startIndex, len, uint40(CKBTxTypes.Script));
    }

    /// @notice         extracts the codeHash from a Script
    /// @param _input   the Script
    /// @return         the codeHash
    function codeHash(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Script) returns (bytes32) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        return _input.index(startIndex, 32);
    }

    /// @notice         extracts the hashType from a Script
    /// @param _input   the Script
    /// @return         the hashType
    function hashType(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Script) returns (uint8) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        return uint8(_input.indexUint(startIndex, 1));
    }

    /// @notice         extracts the args from a Script
    /// @param _input   the Script
    /// @return         the args
    function args(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.Script) returns (address) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        require(20 == len, "invalid contract address in typescript args");
        return _input.indexAddress(startIndex + 4);
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function recipientAddress(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (address) {
        uint256 startIndex = _input.indexLEUint(4, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        require(20 == len, "invalid recipient address in cell data");
        return _input.indexAddress(startIndex + 4);
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function tokenAddress(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (address) {
        uint256 startIndex = _input.indexLEUint(8, 4);
        uint256 len = _input.indexLEUint(startIndex, 4);
        require(20 == len, "invalid token address in cell data");
        return _input.indexAddress(startIndex + 4);
    }

    /// @notice          extracts the outpoint from a CellInput
    /// @param _input    the CellInput
    /// @return          the outpoint as a typed memory
    function tokenAmount(bytes29 _input) internal pure typeAssert(_input, CKBTxTypes.RecipientCellData) returns (uint256) {
        uint256 startIndex = _input.indexLEUint(12, 4);
        uint256 len = _input.len() - startIndex;
        require(16 == len, "invalid token amount in cell data");
        return _input.indexLEUint(startIndex, 16);
    }
}

