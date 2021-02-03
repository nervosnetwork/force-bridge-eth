// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

interface ICKBSpvV3 {
    function proveTxRootExist(bytes calldata txRootProofData) external view returns (bool);
}
