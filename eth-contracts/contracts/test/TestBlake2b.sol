pragma solidity ^0.5.10;
import {CKBCrypto} from "../libraries/CKBCrypto.sol";

contract TestBlake2b {
    function ckbBlake2b(bytes32 left, bytes32 right) public view returns(bytes32) {
        return CKBCrypto.digest(abi.encodePacked(left, right, new bytes(64)), 64);
    }
}
