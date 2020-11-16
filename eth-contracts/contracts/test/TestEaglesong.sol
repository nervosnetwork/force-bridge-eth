pragma solidity ^0.5.10;

import {Eaglesong} from "../Eaglesong.sol";

contract TestEaglesong {
    Eaglesong public songAddr;

    constructor (address _songAddr) public{
        songAddr = Eaglesong(_songAddr);
    }

    function ckbEaglesong(bytes memory data) public returns (bytes32 result) {

        bytes32 high = data[0];
        bytes32 low = data[1];
        return songAddr.hash(high,low);

    }
}
