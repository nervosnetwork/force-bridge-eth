// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "../../../proxy/Delegate.sol";
import "./ProxyTestLayoutV1.sol";
import "./ProxyTestInterfaceV1.sol";
import "../ProxyTestLibrary.sol";


contract ProxyTestLogicV1 is Delegate, ProxyTestLayoutV1, ProxyTestInterfaceV1 {

    function setSlot1(uint256 input) public returns (uint256 old){
        old = slot1_uint256;
        slot1_uint256 = input;
    }

    function getSlot1() external view returns(uint256){
        return getSlot1_internal();
    }

    function getSlot1_internal() internal view returns(uint256){
        return slot1_uint256;
    }

    function fnSlot1() external{
        slot1_uint256 = slot1_uint256 + 1;
    }

    function setSlot2(address input_addr, ProxyTestLibrary.Record memory input_rec) public{
        slot2_map[input_addr] = input_rec;
    }

    function getSlot2(address input) public view returns(ProxyTestLibrary.Record memory){
        return slot2_map[input];
    }
}
