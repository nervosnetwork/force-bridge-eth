// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TokenLocker} from "../TokenLocker.sol";
import {ViewSpv} from "../libraries/ViewSpv.sol";
import {TypedMemView} from "../libraries/TypedMemView.sol";

contract TestTokenLocker is TokenLocker {
    using TypedMemView for bytes;
    using TypedMemView for bytes29;

    function testProveTxRootExist(bytes memory txRootProof, bytes32 targetHistoryTxRoot) public view returns(bool) {
        bytes29 txRootProofView = txRootProof.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxRootProof));
        TokenLocker._proveTxRootExist(txRootProofView, targetHistoryTxRoot);
        return true;
    }

    function testProveTxExist(bytes memory txProof, bytes32 txHash, bytes32 targetTxRoot) public view returns(bool) {
        bytes29 txProofView = txProof.ref(uint40(ViewSpv.SpvTypes.CKBHistoryTxProof));
        TokenLocker._proveTxExist(txProofView, txHash, targetTxRoot);
        return true;
    }


}
