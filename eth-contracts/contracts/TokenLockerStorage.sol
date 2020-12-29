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
import "./TokenLockerLayout.sol";
import "./proxy/Proxy.sol";

contract TokenLockerStorage is Proxy, TokenLockerLayout{
    constructor(
        address ckbSpvAddress,
        uint64 numConfirmations,
        bytes32 recipientCellTypescriptCodeHash,
        uint8 typescriptHashType,
        bytes32 lightClientTypescriptHash,
        bytes32 bridgeCellLockscriptCodeHash,
        address _proxy_admin
    ) Proxy(_proxy_admin){
        ckbSpv_ = ICKBSpv(ckbSpvAddress);
        numConfirmations_ = numConfirmations;
        recipientCellTypescriptCodeHash_ = recipientCellTypescriptCodeHash;
        recipientCellTypescriptHashType_ = typescriptHashType;
        lightClientTypescriptHash_ = lightClientTypescriptHash;
        bridgeCellLockscriptCodeHash_ = bridgeCellLockscriptCodeHash;
    }
}
