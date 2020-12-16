pragma solidity ^0.5.7;

import "./SlotData.sol";

//this is just a normal mapping, but which holds size and you can specify slot
/*
both key and value shouldn't be 0x00
the key must be unique, the value would be whatever

slot
  key --- value
    a --- 1
    b --- 2
    c --- 3
    c --- 4   X   not allowed
    d --- 3
    e --- 0   X   not allowed
    0 --- 9   X   not allowed

*/
contract EnhancedMap is SlotData {

    constructor()public{}

    //set value to 0x00 to delete
    function sysEnhancedMapSet(bytes32 slot, bytes32 key, bytes32 value) internal {
        require(key != bytes32(0x00), "sysEnhancedMapSet, notEmptyKey");
        sysMapSet(slot, key, value);
    }

    function sysEnhancedMapAdd(bytes32 slot, bytes32 key, bytes32 value) internal {
        require(key != bytes32(0x00), "sysEnhancedMapAdd, notEmptyKey");
        require(value != bytes32(0x00), "EnhancedMap add, the value shouldn't be empty");
        require(sysMapGet(slot, key) == bytes32(0x00), "EnhancedMap, the key already has value, can't add duplicate key");
        sysMapSet(slot, key, value);
    }

    function sysEnhancedMapDel(bytes32 slot, bytes32 key) internal {
        require(key != bytes32(0x00), "sysEnhancedMapDel, notEmptyKey");
        require(sysMapGet(slot, key) != bytes32(0x00), "sysEnhancedMapDel, the key doesn't has value, can't delete empty key");
        sysMapSet(slot, key, bytes32(0x00));
    }

    function sysEnhancedMapReplace(bytes32 slot, bytes32 key, bytes32 value) public {
        require(key != bytes32(0x00), "sysEnhancedMapReplace, notEmptyKey");
        require(value != bytes32(0x00), "EnhancedMap replace, the value shouldn't be empty");
        require(sysMapGet(slot, key) != bytes32(0x00), "EnhancedMap, the key doesn't has value, can't replace it");
        sysMapSet(slot, key, value);
    }

    function sysEnhancedMapGet(bytes32 slot, bytes32 key) internal view returns (bytes32){
        require(key != bytes32(0x00), "sysEnhancedMapGet, notEmptyKey");
        return sysMapGet(slot, key);
    }

    function sysEnhancedMapSize(bytes32 slot) internal view returns (uint256){
        return sysMapLen(slot);
    }

}
