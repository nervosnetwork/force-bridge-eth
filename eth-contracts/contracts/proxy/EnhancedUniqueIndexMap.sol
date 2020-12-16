pragma solidity ^0.5.7;

import "./SlotData.sol";

//once you input a value, it will auto generate an index for that
//index starts from 1, 0 means this value doesn't exist
//the value must be unique, and can't be 0x00
//the index must be unique, and can't be 0x00
/*

slot
value --- index
    a --- 1
    b --- 2
    c --- 3
    c --- 4   X   not allowed
    d --- 3   X   not allowed
    e --- 0   X   not allowed

indexSlot = keccak256(abi.encodePacked(keccak256(abi.encodePacked(keccak256(abi.encodePacked(slot))))));
index --- value
    1 --- a
    2 --- b
    3 --- c
    3 --- d   X   not allowed

*/

contract EnhancedUniqueIndexMap is SlotData {

    constructor()public{}

    // slot : value => index
    function sysUniqueIndexMapAdd(bytes32 slot, bytes32 value) internal {

        require(value != bytes32(0x00));

        bytes32 indexSlot = calcIndexSlot(slot);

        uint256 index = uint256(sysMapGet(slot, value));
        require(index == 0, "sysUniqueIndexMapAdd, value already exist");

        uint256 last = sysUniqueIndexMapSize(slot);
        last ++;
        sysMapSet(slot, value, bytes32(last));
        sysMapSet(indexSlot, bytes32(last), value);
    }

    function sysUniqueIndexMapDel(bytes32 slot, bytes32 value) internal {

        //require(value != bytes32(0x00), "sysUniqueIndexMapDel, value must not be 0x00");

        bytes32 indexSlot = calcIndexSlot(slot);

        uint256 index = uint256(sysMapGet(slot, value));
        require(index != 0, "sysUniqueIndexMapDel, value doesn't exist");

        uint256 lastIndex = sysUniqueIndexMapSize(slot);
        require(lastIndex > 0, "sysUniqueIndexMapDel, lastIndex must be large than 0, this must not happen");
        if (index != lastIndex) {

            bytes32 lastValue = sysMapGet(indexSlot, bytes32(lastIndex));
            //move the last to the current place
            //this would be faster than move all elements forward after the deleting one, but not stable(the sequence will change)
            sysMapSet(slot, lastValue, bytes32(index));
            sysMapSet(indexSlot, bytes32(index), lastValue);
        }
        sysMapSet(slot, value, bytes32(0x00));
        sysMapSet(indexSlot, bytes32(lastIndex), bytes32(0x00));
    }

    function sysUniqueIndexMapDelArrange(bytes32 slot, bytes32 value) internal {

        require(value != bytes32(0x00), "sysUniqueIndexMapDelArrange, value must not be 0x00");

        bytes32 indexSlot = calcIndexSlot(slot);

        uint256 index = uint256(sysMapGet(slot, value));
        require(index != 0, "sysUniqueIndexMapDelArrange, value doesn't exist");

        uint256 lastIndex = (sysUniqueIndexMapSize(slot));
        require(lastIndex > 0, "sysUniqueIndexMapDelArrange, lastIndex must be large than 0, this must not happen");

        sysMapSet(slot, value, bytes32(0x00));

        while (index < lastIndex) {

            bytes32 nextValue = sysMapGet(indexSlot, bytes32(index + 1));
            sysMapSet(indexSlot, bytes32(index), nextValue);
            sysMapSet(slot, nextValue, bytes32(index));

            index ++;
        }

        sysMapSet(indexSlot, bytes32(lastIndex), bytes32(0x00));
    }

    function sysUniqueIndexMapReplace(bytes32 slot, bytes32 oldValue, bytes32 newValue) internal {
        require(oldValue != bytes32(0x00), "sysUniqueIndexMapReplace, oldValue must not be 0x00");
        require(newValue != bytes32(0x00), "sysUniqueIndexMapReplace, newValue must not be 0x00");

        bytes32 indexSlot = calcIndexSlot(slot);

        uint256 index = uint256(sysMapGet(slot, oldValue));
        require(index != 0, "sysUniqueIndexMapDel, oldValue doesn't exists");
        require(uint256(sysMapGet(slot, newValue)) == 0, "sysUniqueIndexMapDel, newValue already exists");

        sysMapSet(slot, oldValue, bytes32(0x00));
        sysMapSet(slot, newValue, bytes32(index));
        sysMapSet(indexSlot, bytes32(index), newValue);
    }

    //============================view & pure============================

    function sysUniqueIndexMapSize(bytes32 slot) internal view returns (uint256){
        return sysMapLen(slot);
    }

    //returns index, 0 mean not exist
    function sysUniqueIndexMapGetIndex(bytes32 slot, bytes32 value) internal view returns (uint256){
        return uint256(sysMapGet(slot, value));
    }

    function sysUniqueIndexMapGetValue(bytes32 slot, uint256 index) internal view returns (bytes32){
        bytes32 indexSlot = calcIndexSlot(slot);
        return sysMapGet(indexSlot, bytes32(index));
    }

    // index => value
    function calcIndexSlot(bytes32 slot) internal pure returns (bytes32){
        return calcNewSlot(slot, "index");
    }
}
