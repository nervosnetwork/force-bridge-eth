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
import "./TokenLockerV2Layout.sol";
import "./proxy/Proxy.sol";

contract TokenLockerV2Storage is Proxy, TokenLockerV2Layout{
    constructor(
        address ckbSpvAddress,
        uint64 numConfirmations,
        bytes32 recipientCellTypescriptCodeHash,
        uint8 typescriptHashType,
        bytes32 lightClientTypescriptHash,
        bytes32 bridgeCellLockscriptCodeHash,
        address[] memory validators,
        uint multisigThreshold,
        uint chainId,
        address _proxy_admin
    ) Proxy(_proxy_admin){
        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
        recipientCellTypescriptCodeHash_ = recipientCellTypescriptCodeHash;
        recipientCellTypescriptHashType_ = typescriptHashType;
        lightClientTypescriptHash_ = lightClientTypescriptHash;
        bridgeCellLockscriptCodeHash_ = bridgeCellLockscriptCodeHash;

        // set DOMAIN_SEPARATOR
        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(NAME_712)),
                keccak256(bytes("1")),
                chainId,
                address(this)
            )
        );

        // set validators
        require(validators.length <= VALIDATORS_SIZE_LIMIT, "number of validators exceeds the limit");
        validators_ = validators;
        require(multisigThreshold <= validators.length, "invalid multisigThreshold");
        multisigThreshold_ = multisigThreshold;
    }

}
