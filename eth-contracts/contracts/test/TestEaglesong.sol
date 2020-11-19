pragma solidity ^0.5.10;

import {Eaglesong} from "../Eaglesong.sol";

contract TestEaglesong {
    Eaglesong public songAddr;

    constructor (address _songAddr) public{
        songAddr = Eaglesong(_songAddr);
    }

    function ckbEaglesong(bytes memory data) public returns (bytes32 result) {
        bytes32 high;
        bytes32 low;
        assembly {
            high := mload(add(data, 0x20))
            low := mload(add(0x20, add(data, 0x20)))
        }
        return songAddr.hash(high, low);
    }
}
