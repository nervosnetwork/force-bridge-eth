pragma solidity ^0.5.7;

import "../ProxyTestLibrary.sol";
import "../v1/ProxyTestLayoutV1.sol";

contract ProxyTestLayoutV2 is ProxyTestLayoutV1 {
    bytes32 slot3_bytes32;
}
