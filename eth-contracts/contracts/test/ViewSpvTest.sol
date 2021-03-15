// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "../libraries/TypedMemView.sol";
import {ViewSpv} from "../libraries/ViewSpv.sol";


contract ViewSpvTest {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewSpv for bytes29;

    // members in CKBHistoryTxProof
    function txBlockNumber(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof)).txBlockNumber();
    }

    function historyTxMerkleIndex(bytes memory _input) public pure returns (uint16) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof)).historyTxMerkleIndex();
    }

    function historyWitnessesRoot(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof)).historyWitnessesRoot();
    }

    function historyLemmas(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof)).historyLemmas().clone();
    }

    function rawTransaction(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof)).rawTransaction().clone();
    }

    // members in CkbHistoryTxRootProof
    function initBlockNumber(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)).initBlockNumber();
    }

    function latestBlockNumber(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)).latestBlockNumber();
    }

    function indices(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)).indices().clone();
    }

    function proofLeaves(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)).proofLeaves().clone();
    }

    function txRootLemmas(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof)).txRootLemmas().clone();
    }
}
