// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

interface ICKBChain {
    event BlockHashAdded(
        uint64 indexed blockNumber,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed blockNumber,
        bytes32 blockHash
    );

    function addHeaders(bytes calldata data) external;
}
