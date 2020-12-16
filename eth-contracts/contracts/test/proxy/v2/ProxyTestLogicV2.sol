pragma solidity ^0.5.7;
pragma experimental ABIEncoderV2;

import "../../../proxy/Delegate.sol";
import "../ProxyTestLibrary.sol";
import "./ProxyTestLayoutV2.sol";
import "./ProxyTestInterfaceV2.sol";

contract ProxyTestLogicV2 is Delegate, ProxyTestLayoutV2, ProxyTestInterfaceV2 {

    function setSlot3(bytes32 input) external {
        slot3_bytes32 = input;
    }

    function getSlot3() public returns(bytes32){
        return slot3_bytes32;
    }
}
