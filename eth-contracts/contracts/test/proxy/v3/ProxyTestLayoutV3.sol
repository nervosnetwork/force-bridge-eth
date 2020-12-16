pragma solidity ^0.5.7;

import "../ProxyTestLibrary.sol";
import "../v2/ProxyTestLayoutV2.sol";

//you may use V3 to replace V1, so you want to 'soft' delete V1's layout.
contract ProxyTestLayoutV3 is ProxyTestLayoutV2 {
    uint256[] slot4_dynamic_array;

    uint256[2] slot5_and_6_static_array;

}
