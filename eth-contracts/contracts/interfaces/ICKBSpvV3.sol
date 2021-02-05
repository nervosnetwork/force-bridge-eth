// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

interface ICKBSpvV3 {
    function getHistoryTxRootInfo() external view returns (uint64, uint64, bytes32);
}
