// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "../libraries/TypedMemView.sol";
import {ViewCKB} from "../libraries/ViewCKB.sol";


contract ViewCKBTest {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewCKB for bytes29;

    // members in CellInput
    function since(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewCKB.CKBTypes.CellInput)).since();
    }

    function previousOutput(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewCKB.CKBTypes.CellInput)).previousOutput().clone();
    }

    // members in Script
    function codeHash(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewCKB.CKBTypes.Script)).codeHash();
    }

    function hashType(bytes memory _input) public pure returns (uint8) {
        return _input.ref(uint40(ViewCKB.CKBTypes.Script)).hashType();
    }

    function args(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewCKB.CKBTypes.Script)).args().clone();
    }

    // members in Header
    function rawHeader(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewCKB.CKBTypes.Header)).rawHeader().clone();
    }

    function nonce(bytes memory _input) public pure returns (uint128) {
        return _input.ref(uint40(ViewCKB.CKBTypes.Header)).nonce();
    }

    // members in RawHeader
    function version(bytes memory _input) public pure returns (uint32) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).version();
    }

    function compactTarget(bytes memory _input) public pure returns (uint32) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).compactTarget();
    }

    function timestamp(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).timestamp();
    }

    function blockNumber(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).blockNumber();
    }

    function epoch(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).epoch();
    }

    function parentHash(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).parentHash();
    }

    function transactionsRoot(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).transactionsRoot();
    }

    function unclesHash(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewCKB.CKBTypes.RawHeader)).unclesHash();
    }

    function indexHeaderVec(bytes memory _input, uint256 index) public view returns (bytes memory) {
        return _input.ref(uint40(ViewCKB.CKBTypes.HeaderVec)).indexHeaderVec(index).clone();
    }
}
