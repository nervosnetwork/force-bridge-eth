// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {Eaglesong} from "../Eaglesong.sol";

contract TestEaglesong {
    Eaglesong public songAddr;

    constructor (address _songAddr){
        songAddr = Eaglesong(_songAddr);
    }

    function ckbEaglesong(bytes memory data) public view returns (bytes32 result) {
        bytes32 high;
        bytes32 low;
        // solium-disable-next-line
        assembly {
            high := mload(add(data, 0x20))
            low := mload(add(0x20, add(data, 0x20)))
        }
        return songAddr.hash(high, low);
    }
}
