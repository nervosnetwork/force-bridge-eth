// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./TypedMemView.sol";
import {ViewCKB} from "./ViewCKB.sol";
import {SafeMath} from "./SafeMath.sol";

library ViewSpv {
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;
    using SafeMath for uint;

    enum SpvTypes {
        Unknown,                // 0x0
        CKBTxProof,
        H256,
        H256Array
    }

    // @notice             requires `memView` to be of a specified type
    // @param memView      a 29-byte view with a 5-byte type
    // @param t            the expected type (e.g. CKBTypes.Outpoint, CKBTypes.Script, etc)
    // @return             passes if it is the correct type, errors if not
    modifier typeAssert(bytes29 memView, SpvTypes t) {
        memView.assertType(uint40(t));
        _;
    }

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

    function mockTxHash(bytes29 _input) internal pure typeAssert(_input, SpvTypes.CKBTxProof) returns (bytes32) {
        return bytes32(0);
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

}
