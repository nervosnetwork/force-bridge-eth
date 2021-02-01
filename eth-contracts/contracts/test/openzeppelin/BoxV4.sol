// contracts/BoxV2.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract BoxV4 {
    uint256 private value;

    struct Header {
        uint64 number;
        bytes32 hash;
    }

    Header public currentHeader;

    // Emitted when the stored value changes
    event ValueChanged(uint256 newValue);

    // Stores a new value in the contract
    function store(uint256 newValue) public {
        value = newValue;
        emit ValueChanged(newValue);
    }

    // Reads the last stored value
    function retrieve() public view returns (uint256) {
        return value;
    }

    // Increments the stored value by 1
    function increment() public {
        value = value + 2000;
        emit ValueChanged(value);
    }

    function setHeader(uint64 number, bytes32 hash) public {
        currentHeader.number = number;
        currentHeader.hash = hash;
    }
}
