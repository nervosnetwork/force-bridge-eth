// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

contract TestConstants {
    function calcSetNewCkbSpvTypehash() public view returns(bytes32) {
        bytes32 SET_NEW_CKB_SPV_TYPEHASH = keccak256("SetNewCkbSpv(address newSpvAddress,uint256 nonce)");
        return SET_NEW_CKB_SPV_TYPEHASH;
    }
}
