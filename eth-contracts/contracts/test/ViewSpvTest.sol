// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "../libraries/TypedMemView.sol";
import {ViewSpv} from "../libraries/ViewSpv.sol";


contract ViewSpvTest {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;
    using ViewSpv for bytes29;

    // members in CKBTxProof
    function txMerkleIndex(bytes memory _input) public pure returns (uint16) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBTxProof)).txMerkleIndex();
    }

    function blockNumber(bytes memory _input) public pure returns (uint64) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBTxProof)).spvBlockNumber();
    }

    function blockHash(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBTxProof)).blockHash();
    }

    function txHash(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBTxProof)).txHash();
    }

    function witnessesRoot(bytes memory _input) public pure returns (bytes32) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBTxProof)).witnessesRoot();
    }

    function lemmas(bytes memory _input) public view returns (bytes memory) {
        return _input.ref(uint40(ViewSpv.SpvTypes.CKBTxProof)).lemmas().clone();
    }
}
