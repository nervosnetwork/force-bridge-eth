// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

interface ICKBChainV3 {
    event HistoryTxRootAdded(
        uint64 indexed startBlockNumber,
        uint64 indexed endBlockNumber,
        bytes32 HistoryTxRoot
    );

    function addHistoryTxRoot(uint64 _initBlockNumber, uint64 _latestBlockNumber, bytes32 _historyTxRoot, bytes calldata signatures) external;
}
