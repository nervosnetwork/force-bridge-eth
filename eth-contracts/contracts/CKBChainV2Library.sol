// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

contract CKBChainV2Library{


    // Minimal information about the submitted block.
    struct BlockHeader {
        uint64 number;
        uint64 epoch;
        uint256 difficulty;
        uint256 totalDifficulty;
        bytes32 parentHash;
    }
}
