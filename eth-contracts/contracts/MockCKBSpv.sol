pragma solidity ^0.5.10;

import {ICKBSpv} from "./interfaces/ICKBSpv.sol";

contract MockCKBSpv is ICKBSpv {
    function proveTxExist(bytes calldata txProofData, uint64 numConfirmations) external view returns(bool) {
        return true;
    }
}
