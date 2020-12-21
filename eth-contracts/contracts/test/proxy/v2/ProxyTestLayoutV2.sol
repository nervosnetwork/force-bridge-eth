// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "../ProxyTestLibrary.sol";
import "../v1/ProxyTestLayoutV1.sol";

contract ProxyTestLayoutV2 is ProxyTestLayoutV1 {
    bytes32 slot3_bytes32;
}
