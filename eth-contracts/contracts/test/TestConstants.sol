pragma solidity ^0.5.10;

contract TestConstants {
    function calcSetNewCkbSpvTypehash() public view returns(bytes32) {
        bytes32 SET_NEW_CKB_SPV_TYPEHASH = keccak256("SetNewCkbSpv(address newSpvAddress,uint256 nonce)");
        return SET_NEW_CKB_SPV_TYPEHASH;
    }
}
