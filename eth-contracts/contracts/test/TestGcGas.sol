pragma solidity ^0.5.10;
import "hardhat/console.sol";

contract TestGcGas {

    mapping(uint64 => bytes32) canonicalHeaderHashes;
    mapping(bytes32 => uint64) queueIndex;

    function setStorage() external {
        for (uint64 i = 0; i < 100; i++) {
            canonicalHeaderHashes[i] = bytes32(0x1111111111111111111111111111111111111111111111111111111111111111);
        }
    }

    function testGcGas() external {
        for (uint64 i = 100; i < 200; i++) {
            canonicalHeaderHashes[i] = bytes32(0x1111111111111111111111111111111111111111111111111111111111111111);
        }

        for (uint64 i = 0; i < 100; i++) {
            delete canonicalHeaderHashes[i];
        }
    }

    function testQueueGas() external {
        for (uint64 i = 0; i < 100; i++) {
            uint64 index = (i + 1) % 100;
            canonicalHeaderHashes[index] = bytes32(0x2222222222222222222222222222222222222222222222222222222222222222);
            queueIndex[bytes32(0x2222222222222222222222222222222222222222222222222222222222222222)] = index;
        }
    }
}
