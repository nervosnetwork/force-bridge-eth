pragma solidity ^0.5.7;

import "../ProxyTestLibrary.sol";

contract ProxyTestLayoutV1 {
    uint256 slot1_uint256;

    mapping(address => ProxyTestLibrary.Record) slot2_map;
}
