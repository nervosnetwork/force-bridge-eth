// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "../ProxyTestLibrary.sol";

contract ProxyTestLayoutV1 {
    uint256 slot1_uint256;

    mapping(address => ProxyTestLibrary.Record) slot2_map;
}
