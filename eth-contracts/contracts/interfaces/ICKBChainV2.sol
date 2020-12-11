pragma solidity ^0.5.10;

interface ICKBChainV2 {
    event BlockHashAdded(
        uint64 indexed blockNumber,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed blockNumber,
        bytes32 blockHash
    );

    function addHeaders(bytes calldata data, bytes calldata signatures) external;
}
