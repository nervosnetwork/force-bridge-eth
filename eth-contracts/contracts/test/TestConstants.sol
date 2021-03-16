// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

contract TestConstants {
    function calcTypehash() public view returns(bytes32, bytes32) {
        bytes32 ADD_HISTORY_TX_ROOT_TYPEHASH = keccak256("AddHistoryTxRoot(uint64 startBlockNumber,uint64 endBlockNumber,bytes32 historyTxRoot)");
        bytes32 SET_NEW_VALIDATORS_TYPEHASH = keccak256("SetNewValidators(address[] validators,uint256 multisigThreshold)");
        return (ADD_HISTORY_TX_ROOT_TYPEHASH, SET_NEW_VALIDATORS_TYPEHASH);
    }
}
