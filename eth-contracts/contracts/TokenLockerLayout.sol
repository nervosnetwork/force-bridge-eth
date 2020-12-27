// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./interfaces/ICKBSpv.sol";

contract TokenLockerV2Layout{

    //constants take no space, declare them at the begin
    //==============================================================================


    //==============================================================================
    uint8 public recipientCellTypescriptHashType_;
    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;
    bytes32 public recipientCellTypescriptCodeHash_;
    bytes32 public lightClientTypescriptHash_;
    bytes32 public bridgeCellLockscriptCodeHash_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;
}
