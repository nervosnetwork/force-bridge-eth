pragma solidity ^0.5.7;
pragma experimental ABIEncoderV2;

import "../../proxy/Proxy.sol";
import "./v1/ProxyTestLayoutV1.sol";

contract ProxyTestStorage is Proxy, ProxyTestLayoutV1 {

    constructor(uint256 _slot1_uint256, address _sysAdmin) Proxy(_sysAdmin)public {
        slot1_uint256 = _slot1_uint256;
    }
}
