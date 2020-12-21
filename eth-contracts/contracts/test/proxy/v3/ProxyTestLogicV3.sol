// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "../../../proxy/Delegate.sol";
import "../ProxyTestLibrary.sol";
import "./ProxyTestLayoutV3.sol";
import "./ProxyTestInterfaceV3.sol";

//use V3 to replace V2
contract ProxyTestLogicV3 is Delegate, ProxyTestLayoutV3, ProxyTestInterfaceV3 {

    //you replace the fnSlot1 function
    function fnSlot1() external{
        slot1_uint256 = slot1_uint256 * 2;
    }

    //!!!you 'soft' delete the other functions from logicV1
    //typically, you keep the slot1 and replace fnSlot1()
    //and you 'soft' delete:
    //setSlot1()
    //getSlot1()
    //and you 'soft' delete slot2 and its functions
    //slot2
    //setSlot2()
    //getSlot2()

    function getSlot1() external view returns(uint256){
        return slot1_uint256;
    }

    function pushSlot4(uint256 input) public{
        slot4_dynamic_array.push(input);
    }

    function getSlot4(uint256 input) public view returns(uint256){
        return slot4_dynamic_array[input];
    }

    function setSlot5(uint256 left, uint256 right) public {
        slot5_and_6_static_array[0] = left;
        slot5_and_6_static_array[1] = right;
    }

    function getSlot5() public view returns(uint256 left, uint256 right){
        left = slot5_and_6_static_array[0];
        right = slot5_and_6_static_array[1];
    }

}
