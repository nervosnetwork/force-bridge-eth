// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

interface ICKBSpv {
    // Number of `NumConfirmations` that applications can use to consider the transaction safe.
    // For most use cases 25 should be enough, for super safe cases it should be 500.
    function proveTxExist(bytes calldata txProofData, uint64 numConfirmations) external view returns(bool);
}
