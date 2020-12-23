// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./interfaces/ICKBSpv.sol";

contract TokenLockerV2Layout{

    //constants take no space, declare them at the begin
    //==============================================================================

    // refer to https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md
    uint public constant SIGNATURE_SIZE = 65;
    uint public constant VALIDATORS_SIZE_LIMIT = 20;
    string public constant NAME_712 = "Force Bridge TokenLocker";
    // SET_NEW_CKB_SPV_TYPEHASH = keccak256("SetNewCkbSpv(address newSpvAddress,uint256 nonce)");
    bytes32 public constant SET_NEW_CKB_SPV_TYPEHASH = 0x27ed7da1c419c8aa5f5c5f74f651895eaf8dab9831123b22248827a707d44bb1;


    //==============================================================================

    uint64 public numConfirmations_;
    ICKBSpv public ckbSpv_;
    bytes32 public recipientCellTypescriptCodeHash_;
    uint8 public recipientCellTypescriptHashType_;
    bytes32 public bridgeCellLockscriptCodeHash_;

    // txHash -> Used
    mapping(bytes32 => bool) public usedTx_;

    bytes32 public DOMAIN_SEPARATOR;
    uint public currentSetNewCkbSpvNonce;
    // if the number of verified signatures has reached `multisigThreshold_`, validators approve the tx
    uint public multisigThreshold_;
    address[] validators_;
}
