pragma solidity ^0.7.4;
import {Hamstersong} from "../Hamstersong.sol";

contract TestHamstersong {
    address public songAddr;

    constructor(address hamstersongAddr) {
        songAddr = hamstersongAddr;
    }

    function ckbEaglesong(bytes memory data) public returns (bytes32 result) {
        bytes memory payload = abi.encodePacked(
            hex"aa6d7de4",
            data
        );

        (bool success, bytes memory returnData) = songAddr.call(payload);

        assembly {
            result := mload(add(returnData, 0x20))
        }
    }

}
