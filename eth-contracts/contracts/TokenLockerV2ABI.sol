// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./interfaces/IERC20.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {TypedMemView} from "./libraries/TypedMemView.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {CKBTxView} from "./libraries/CKBTxView.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {Address} from "./libraries/Address.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import {MultisigUtils} from "./libraries/MultisigUtils.sol";

abstract contract TokenLockerV2ABI {
    event Locked(
        address indexed token,
        address indexed sender,
        uint256 lockedAmount,
        uint256 bridgeFee,
        bytes recipientLockscript,
        bytes replayResistOutpoint,
        bytes sudtExtraData
    );

    event Unlocked(
        address indexed token,
        address indexed recipient,
        address indexed sender,
        uint256 receivedAmount,
        uint256 bridgeFee
    );

    event NewCkbSpv(
        address ckbSpvAddress,
        uint256 nonce
    );

    /**
     * @notice  if addr is not one of validators_, return validators_.length
     * @return  index of addr in validators_
     */
    function getIndexOfValidators(address user) virtual internal view returns (uint) ;

    /**
     * @notice             @dev signatures are a multiple of 65 bytes and are densely packed.
     * @param msgHash      sth. which signers sign
     * @param signatures   The signatures bytes array
     * @param threshold    check number of verified signatures >= `threshold`, signatures are approved by validators
     */
    function validatorsApprove(bytes32 msgHash, bytes memory signatures, uint threshold) virtual public view;

    function setNewCkbSpv(
        address newSpvAddress,
        uint nonce,
        bytes memory signatures
    ) virtual public;

    function lockETH(
        uint256 bridgeFee,
        bytes memory recipientLockscript,
        bytes memory replayResistOutpoint,
        bytes memory sudtExtraData
    ) virtual public payable;

    // before lockToken, user should approve -> TokenLocker Contract with 0xffffff token
    function lockToken(
        address token,
        uint256 amount,
        uint256 bridgeFee,
        bytes memory recipientLockscript,
        bytes memory replayResistOutpoint,
        bytes memory sudtExtraData
    ) virtual public;

    function unlockToken(bytes memory ckbTxProof, bytes memory ckbTx) virtual public;

    // TODO refund function

    function decodeBurnResult(bytes memory ckbTx) virtual public view returns (
        uint256 bridgeAmount,
        uint256 bridgeFee,
        address token,
        address recipient
    );
}
