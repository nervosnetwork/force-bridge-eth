// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./interfaces/ICKBSpv.sol";

abstract contract CKBChainV2ABI is ICKBSpv{
    event BlockHashAdded(
        uint64 indexed blockNumber,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed blockNumber,
        bytes32 blockHash
    );

    function addHeaders(bytes calldata data, bytes calldata signatures) virtual external;
}
