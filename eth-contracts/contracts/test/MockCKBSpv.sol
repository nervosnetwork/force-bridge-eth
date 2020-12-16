// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {ICKBSpv} from "../interfaces/ICKBSpv.sol";

contract MockCKBSpv is ICKBSpv {
    function proveTxExist(bytes calldata _txProofData, uint64 _numConfirmations) override external view returns(bool) {
        return true;
    }
}
