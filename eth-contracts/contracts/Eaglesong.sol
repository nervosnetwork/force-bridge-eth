// SPDX-License-Identifier: MIT

/*
The Times 11/Nov/2020 Developers on brink of bailout for CKB Eaglesong Hash function.

A Golden Hamster saves them!
*/
pragma solidity ^0.8.0;

contract Eaglesong {

    /*
    for test
    0x412f9cac + 0x11223344556677889900112233445566778899001122334455667788990011223344556677889900112233445566778800000000000000000000000000000000
    0x412f9cac11223344556677889900112233445566778899001122334455667788990011223344556677889900112233445566778800000000000000000000000000000000

    0x412f9cac + 0x6162636465666768696a6b6c6d6e6f707172737475767778797a303132333435363738396162636465666768696a6b6c00000000000000000000000000000000
    0x412f9cac6162636465666768696a6b6c6d6e6f707172737475767778797a303132333435363738396162636465666768696a6b6c00000000000000000000000000000000
    */

    // 0x412f9cac1faebe3bbfeaf64d8f0f368d2937e0942f0b863185954f813b09c833 == abi = keccak256("hash(bytes32,bytes32)")
    // 0x412f9cac
    // the param is bytes32 high, bytes32 low
    function hash(bytes32,bytes32) pure external returns(bytes32){
        assembly{
        // force take over the memory management
        // allocate 0x20 bytes, 0x80+0x20=0xA0
            mstore(0x40, 0xA0)

        // reject not ckbinput
            if eq(eq(calldatasize(), 68),0){

            //revert (0, 0)
                let size := calldatasize()
                revert(0x80,0x00)
            }

        // 0x00 0x6162636465666768696a6b6c6d6e6f70
        // 0x10 0x7172737475767778797a303132333435

        // 0x20 0x363738396162636465666768696a6b6c
        // 0x30 0x00000006000000000000000000000000

        // stack ->  high || low
        // stack uses 2 slot for 256*2 = 512 bits
        // high/r || low/c = state_vector

            let low := byte(0x20,0x00)
            let high := byte(0x20,0x00)

        // these are for intermediate computation
            let new_low := byte(0x20, 0x00)
            let new_high := byte(0x20, 0x00)

        // a chunk is 256 bits, as the 'r', a.k.a. rate, means
        // start 2 times for 2 chunks


        //=================================================ABSORB_CHUNK===========================================================

            for {let chunk := 0} lt(chunk, 2) {chunk := add(chunk, 1)}{
            /*
                chunk 0
            */
                switch chunk
                case 0{
                    high := xor(high, calldataload(0x04))
                }
                case 1{

                    high := xor(high, or(calldataload(0x24),0x000000000000000000000000000000000000006000000000000000000000000))
                }
                default{
                    revert(0x80,0x00)
                }

            // here Permutation begins

            // one permutation has independent 43 round
                for {let round := 0} lt(round, 43) {round := add(round, 1)}{

                    {//bit_matrix
                        new_low := byte(0x20, 0x00)
                        new_high := byte(0x20, 0x00)

                    /*bit_matrix = [
                             0 1 2 3 4 5 6 7|8 9 a b c d e f
                        0	[1 1 1 1 0 1 0 1 1 1 1 1 0 0 0 1]
                        1   [0 1 1 1 1 0 1 0 1 1 1 1 1 0 0 1]
                        2   [0 0 1 1 1 1 0 1 0 1 1 1 1 1 0 1]
                        3   [0 0 0 1 1 1 1 0 1 0 1 1 1 1 1 1]
                        4   [1 1 1 1 1 0 1 0 1 0 1 0 1 1 1 0]
                        5   [1 0 0 0 1 0 0 0 1 0 1 0 0 1 1 1]
                        6   [1 0 1 1 0 0 0 1 1 0 1 0 0 0 1 0]
                        7   [1 0 1 0 1 1 0 1 0 0 1 0 0 0 0 1]
                        8   [0 1 0 1 0 1 1 0 1 0 0 1 0 0 0 1]
                        9   [0 0 1 0 1 0 1 1 0 1 0 0 1 0 0 1]
                        a   [0 0 0 1 0 1 0 1 1 0 1 0 0 1 0 1]
                        b   [0 0 0 0 1 0 1 0 1 1 0 1 0 0 1 1]
                        c   [1 1 1 1 0 0 0 0 1 0 0 1 1 0 0 0]
                        d   [0 1 1 1 1 0 0 0 0 1 0 0 1 1 0 0]
                        e   [0 0 1 1 1 1 0 0 0 0 1 0 0 1 1 0]
                        f   [1 1 1 0 1 0 1 1 1 1 1 0 0 0 1 1]
                    ]*/



                    // alloc a item for temp usage of 32 bits
                    // we use the lowest 32 bits of 256 bits
                        let output_vector_32 := byte(0x20, 0x00)// for output_vector[j]
                        let state_vector_32 := byte(0x20, 0x00)// for dup of original state_vector

                    // row 0,1 0 0 0 1 1 1 1 0 0 0 0 1 0 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))


                    // column 1
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column 2
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column 3
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_high := or(new_high, shl(0xE0, output_vector_32))


                    // row 1,1 1 0 0 1 0 0 0 1 0 0 0 1 1 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))


                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column 3
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_high := or(new_high, shl(0xC0, output_vector_32))

                    // row 2,1 1 1 0 1 0 1 1 0 1 0 0 1 1 1 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_high := or(new_high, shl(0xA0, output_vector_32))

                    // row 3,1 1 1 1 1 0 1 0 1 0 1 0 1 1 1 0
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                        new_high := or(new_high, shl(0x80, output_vector_32))

                    // row 4,0 1 1 1 1 1 0 1 0 1 0 1 0 1 1 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_high := or(new_high, shl(0x60, output_vector_32))

                    // row 5,1 0 1 1 0 0 0 1 1 0 1 0 0 0 1 0
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                        new_high := or(new_high, shl(0x40, output_vector_32))

                    // row 6,0 1 0 1 1 0 0 0 1 1 0 1 0 0 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_high := or(new_high, shl(0x20, output_vector_32))

                    // row 7,1 0 1 0 0 0 1 1 0 1 1 0 0 0 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column 4
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_high := or(new_high, output_vector_32)

                    // row 8,1 1 0 1 1 1 1 0 1 0 1 1 1 0 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_low := or(new_low, shl(0xE0, output_vector_32))


                    // row 9,1 1 1 0 0 0 0 0 0 1 0 1 0 1 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column 4
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_low := or(new_low, shl(0xC0, output_vector_32))

                    // row a,1 1 1 1 1 1 1 1 0 0 1 0 0 0 1 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_low := or(new_low, shl(0xA0, output_vector_32))

                    // row b,1 1 1 1 0 0 0 0 1 0 0 1 1 0 0 0
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                        new_low := or(new_low, shl(0x80, output_vector_32))

                    // row c,0 1 1 1 1 0 0 0 0 1 0 0 1 1 0 0
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                        new_low := or(new_low, shl(0x60, output_vector_32))

                    // row d,0 0 1 1 1 1 0 0 0 0 1 0 0 1 1 0
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 1
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))*/

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                        new_low := or(new_low, shl(0x40, output_vector_32))

                    // row e,0 0 0 1 1 1 1 0 0 0 0 1 0 0 1 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 1
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column 2
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column 7
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))*/

                    // column 8
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))*/

                    // column 9
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))*/

                    // column a
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))*/

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_low := or(new_low, shl(0x20, output_vector_32))

                    // row f,1 1 1 1 0 1 0 1 1 1 1 1 0 0 0 1
                        output_vector_32 := byte(0x20, 0x00)

                    // column 0
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 1
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column 2
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column 3
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column 4
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column 5
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, shr(0x40,and(state_vector_32, 0x0000000000000000000000000000000000000000ffffffff0000000000000000)))

                    // column 6
                    /*state_vector_32 := high
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column 7
                        state_vector_32 := high
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                    // column 8
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xE0, state_vector_32))

                    // column 9
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xC0, and(state_vector_32, 0x00000000ffffffff000000000000000000000000000000000000000000000000)))

                    // column a
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0xA0, and(state_vector_32, 0x0000000000000000ffffffff0000000000000000000000000000000000000000)))

                    // column b
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, shr(0x80, and(state_vector_32, 0x000000000000000000000000ffffffff00000000000000000000000000000000)))

                    // column c
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x60,and(state_vector_32,0x00000000000000000000000000000000ffffffff000000000000000000000000)))*/

                    // column d
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x40, and(state_vector_32,0x0000000000000000000000000000000000000000ffffffff0000000000000000)))*/

                    // column e
                    /*state_vector_32 := low
                    output_vector_32 := xor(output_vector_32,shr(0x20, and(state_vector_32,0x000000000000000000000000000000000000000000000000ffffffff00000000)))*/

                    // column f
                        state_vector_32 := low
                        output_vector_32 := xor(output_vector_32, and(state_vector_32, 0x00000000000000000000000000000000000000000000000000000000ffffffff))

                        new_low := or(new_low, output_vector_32)

                        low := new_low
                        high := new_high

                    // bit matrix
                    }


                    {// circulant multiplication
                        new_low := byte(0x20, 0x00)
                        new_high := byte(0x20, 0x00)

                    /*coefficients = [
                        [0, 2, 4],
                        [0, 13, 22],
                        [0, 4, 19],
                        [0, 3, 14],
                        [0, 27, 31],
                        [0, 3, 8],
                        [0, 17, 26],
                        [0, 3, 12],
                        [0, 18, 22],
                        [0, 12, 18],
                        [0, 4, 7],
                        [0, 4, 31],
                        [0, 12, 27],
                        [0, 7, 17],
                        [0, 7, 8],
                        [0, 1, 13]
                    ];*/



                    // alloc a item for temp usage of 32 bits
                    // we use the lowest 32 bits of 256 bits
                        let output_vector_32 := byte(0x20, 0x00)// for output_vector[j]
                        let state_vector := byte(0x20, 0x00)

                    // row 0,0, 2, 4
                        state_vector := shr(0xE0,high)

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(2,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(30,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(4,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(28,state_vector)
                        ))


                        new_high := or(new_high, shl(0xE0, output_vector_32))


                    // row 1,0, 13, 22

                        state_vector := shr(0xC0,and(high, 0x00000000ffffffff000000000000000000000000000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(13,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(19,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(22,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(10,state_vector)
                        ))

                        new_high := or(new_high, shl(0xC0, output_vector_32))

                    // row 2,0, 4, 19

                        state_vector := shr(0xA0,and(high, 0x0000000000000000ffffffff0000000000000000000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(4,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(28,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(19,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(13,state_vector)
                        ))

                        new_high := or(new_high, shl(0xA0, output_vector_32))

                    // row 3,0, 3, 14

                        state_vector := shr(0x80,and(high, 0x000000000000000000000000ffffffff00000000000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(3,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(29,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(14,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(18,state_vector)
                        ))

                        new_high := or(new_high, shl(0x80, output_vector_32))

                    // row 4,0, 27, 31

                        state_vector := shr(0x60,and(high, 0x00000000000000000000000000000000ffffffff000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(27,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(5,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(31,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(1,state_vector)
                        ))

                        new_high := or(new_high, shl(0x60, output_vector_32))

                    // row 5,0, 3, 8

                        state_vector := shr(0x40,and(high, 0x0000000000000000000000000000000000000000ffffffff0000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(3,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(29,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(8,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(24,state_vector)
                        ))

                        new_high := or(new_high, shl(0x40, output_vector_32))

                    // row 6,0, 17, 26

                        state_vector := shr(0x20,and(high, 0x000000000000000000000000000000000000000000000000ffffffff00000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(17,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(15,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(26,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(6,state_vector)
                        ))

                        new_high := or(new_high, shl(0x20, output_vector_32))

                    // row 7,0, 3, 12

                        state_vector := and(high, 0x00000000000000000000000000000000000000000000000000000000ffffffff)

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(3,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(29,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(12,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(20,state_vector)
                        ))

                        new_high := or(new_high, output_vector_32)

                    // row 8,0, 18, 22

                        state_vector := shr(0xE0,low)

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(18,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(14,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(22,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(10,state_vector)
                        ))

                        new_low := or(new_low, shl(0xE0, output_vector_32))

                    // row 9,0, 12, 18

                        state_vector := shr(0xC0,and(low, 0x00000000ffffffff000000000000000000000000000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(12,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(20,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(18,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(14,state_vector)
                        ))

                        new_low := or(new_low, shl(0xC0, output_vector_32))

                    // row a,0, 4, 7

                        state_vector := shr(0xA0,and(low, 0x0000000000000000ffffffff0000000000000000000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(4,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(28,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(7,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(25,state_vector)
                        ))

                        new_low := or(new_low, shl(0xA0, output_vector_32))

                    // row b,0, 4, 31

                        state_vector := shr(0x80,and(low, 0x000000000000000000000000ffffffff00000000000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(4,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(28,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(31,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(1,state_vector)
                        ))

                        new_low := or(new_low, shl(0x80, output_vector_32))

                    // row c,0, 12, 27

                        state_vector := shr(0x60,and(low, 0x00000000000000000000000000000000ffffffff000000000000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(12,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(20,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(27,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(5,state_vector)
                        ))

                        new_low := or(new_low, shl(0x60, output_vector_32))

                    // row d,0, 7, 17

                        state_vector := shr(0x40,and(low, 0x0000000000000000000000000000000000000000ffffffff0000000000000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(7,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(25,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(17,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(15,state_vector)
                        ))

                        new_low := or(new_low, shl(0x40, output_vector_32))

                    // row e,0, 7, 8

                        state_vector := shr(0x20,and(low, 0x000000000000000000000000000000000000000000000000ffffffff00000000))

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(7,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(25,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(8,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(24,state_vector)
                        ))

                        new_low := or(new_low, shl(0x20, output_vector_32))

                    // row f,0, 1, 13

                        state_vector := and(low, 0x00000000000000000000000000000000000000000000000000000000ffffffff)

                        output_vector_32 := state_vector

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(1,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(31,state_vector)
                        ))

                        output_vector_32 := xor(output_vector_32,or(
                        and(shl(13,state_vector),0x00000000000000000000000000000000000000000000000000000000ffffffff),
                        shr(19,state_vector)
                        ))

                        new_low := or(new_low, output_vector_32)

                    // finish
                        low := new_low
                        high := new_high

                    // circulant multiplication
                    }

                    {//Injection of Constants

                        switch round

                        case 0{
                            high := xor(high,0x6e9e40ae71927c029a13d3b1daec32ad3d8951cfe1c9fe9ab806b54cacbbf417)
                            low := xor(low,0xd3622b3ba082762a9edcf1c0a9bada777f91e46ccb0f6e4f265d9241b7bdeab0)
                        }
                        case 1{
                            high := xor(high,0x6260c9e6ff50dd2a9036aa71ce161879d1307cdf89e456dff83133e265f55c3d)
                            low := xor(low,0x94871b01b5d204cd583a32645e1659574cbda964675fca47f4a3033e2a417322)
                        }
                        case 2{
                            high := xor(high,0x3b61432f7f5532f2b609973b1a79523931b477c9d2949d28789697120eb87b6e)
                            low := xor(low,0x7e11d22dccee88bdeed07eb8e5563a81e7cb6bcf25de953e4d05653a0b831557)
                        }
                        case 3{
                            high := xor(high,0x94b9cd7713f01579794b4a4a67e7c7dcc456d8d459689c9b668456d722d2a2e1)
                            low := xor(low,0x38b3a8280315ac3c438d681eab7109c597ee19a8de062b2e2c76c47b0084456f)
                        }
                        case 4{
                            high := xor(high,0x908f0fd3a646551f3e826725d521788e9f01c2b093180cdc92ea1df8431a9aae)
                            low := xor(low,0x7c2ea356da33ad034692689366bde7d7b501cc751f6e8a41685250f43bb1f318)
                        }
                        case 5{
                            high := xor(high,0xaf238c04974ed2ec5b159e49d526f8bf120856263e2432a96bd20c481f1d59da)
                            low := xor(low,0x18ab106880f83cf82c8c11c07d5480350ff675c3fed160bf74bbbb24d98e006b)
                        }
                        case 6{
                            high := xor(high,0xdeaa47eb05f2179e437b0b71a7c95f8f00a99d3b3fc3c44472686f8e00fd01a9)
                            low := xor(low,0xdedc0787c6af76267012fe76f2a5f7ce9a7b2eda5e57fcf24da0d4ad5c63b155)
                        }
                        case 7{
                            high := xor(high,0x34117375d4134c112ea774355278b6deab522c4cbc8fc702c94a09e4ebb93a9e)
                            low := xor(low,0x91ecb65e4c52ecc68703bb52cb2d60aa30a0538a1514f10b157f63293429dc3d)
                        }
                        case 8{
                            high := xor(high,0x5db73eb2a7a1a9697286bd240df6881e3785ba5fcd04623a02758170d827f556)
                            low := xor(low,0x99d9519184457eb158a7fb22d2967c5f4f0c33f64a02099ae090482194124036)
                        }
                        case 9{
                            high := xor(high,0x496a031b780b69c4cf1a492787a119b8cdfaf4f84cf9cd0f27c96a846d11117e)
                            low := xor(low,0x7f8cf84774ceede5c88905e6602158417172875a736e993a010aa53c43d53c2b)
                        }
                        case 10{
                            high := xor(high,0xf0d91a930d983b56f816663ce5d133630a61737c09d5115083a5ac2f3e884905)
                            low := xor(low,0x7b01aeb5600a6ea7b7678f7b72b38977068018f2ce6ae45b29188aa8e5a0b1e9)
                        }
                        case 11{
                            high := xor(high,0xc04c2b868bd14d75648781f3dbae1e0addcdd8aeab4d81a3446baaba1cc0c19d)
                            low := xor(low,0x17be4f9082c0e65d676f9c955c708db26fd4c867a5106ef019dde49d78182f95)
                        }
                        case 12{
                            high := xor(high,0xd089cd81a32e98febe306c826cd83d8c037f1bde0b15722deddc1e2293c76559)
                            low := xor(low,0x8a2f571b92cc81b4021b747767523904c95dbcccac17ee9d944e46bc0781867e)
                        }
                        case 13{
                            high := xor(high,0xc854dd9d26e2c30c858c04166d397708ebe29c58c80ced86d496b4abbe45e6f5)
                            low := xor(low,0x10d24706acf8187a96f523cb2227e14378c365644643adc24729d97acff93e0d)
                        }
                        case 14{
                            high := xor(high,0x25484bbd91c6798e95f773f4442046752eda57ba06d313efeeaa44662dfa7530)
                            low := xor(low,0xa8af0c9b39f1535e0cc2b7bd38a76c0e4f41071dcdaf247549a6eff801621748)
                        }
                        case 15{
                            high := xor(high,0x36ebacabbd6d9a2944d1cd6540815dfd55fa5a1a87cce9e9ae559b45d76b4c26)
                            low := xor(low,0x637d60adde29f5f997491cbbfb350040ffe7f997201c9dcde61320e9a90987a3)
                        }
                        case 16{
                            high := xor(high,0xe24afa8361c1e6fccc87ff62f1c9d8fa4fd0454690ecc76e46e456b9305dceb8)
                            low := xor(low,0xf627e68c2d286815c705bbfd101b6df3892dae62d5b7fb44ea1d5c945332e3cb)
                        }
                        case 17{
                            high := xor(high,0xf856f88ab341b0e928408d9d5421bc17eb9af9bc602371c567985a91d774907f)
                            low := xor(low,0x7c4d697d9370b0b86ff5cebb7d465744674ceac0ea9102fc0de94784c793de69)
                        }
                        case 18{
                            high := xor(high,0xfe599bb1c6ad952f6d6ca9c3928c3f91f9022f0524a164dce5e98cd37649efdb)
                            low := xor(low,0x6df3bcdb5d1e9ff117f5d010e2686ea16eac77fe7bb5c58588d90cbb18689163)
                        }
                        case 19{
                            high := xor(high,0x67c9efa5c0b76d9b960efbabbd87280770f4c47456c29d20d1541d1588137033)
                            low := xor(low,0xe3f02b3eb6d9b28d53a077baeedcd29ea50a6c1d12c2801e52ba335b35984614)
                        }
                        case 20{
                            high := xor(high,0xe2599aa8af94ed1dd90d4767202c7d0777bec4f4fa71bc80fc5c8b768d0fbbfc)
                            low := xor(low,0xda366dc68b32a0c71b36f7fc6642dcbc6fe7e7248b5fa782c42274043a7d1da7)
                        }
                        case 21{
                            high := xor(high,0x517ed6588a18df6d3e5c9b231fbd51ef1470601d3400389c676b065d8864ad80)
                            low := xor(low,0xea6f1a9c2db484e1608785f08dd384af69d26699409c4e1677f9986a7f491266)
                        }
                        case 22{
                            high := xor(high,0x883ea6cfeaa06072fa2e5db5352594b49156bb89a2fbbbfbac3989c76e2422b1)
                            low := xor(low,0x581f35601009a9b57e5ad9cda9fc0a6e43e5998e7f8778f9f038f8e15415c2e8)
                        }
                        case 23{
                            high := xor(high,0x6499b731b82389ae05d4d8190f06440ef1735aa0986430ee47ec952cbf149cc5)
                            low := xor(low,0xb3cb2cb63f41e8c2271ac51b48ac5dedf76a0469717bba4d4f5c90d63b74f756)
                        }
                        case 24{
                            high := xor(high,0x1824110aa4fd43e31eb0507ca9375c08157c59a70cad8f51d66031a0abb5343f)
                            low := xor(low,0xe533fa431996e2bbd7953a71d2529b9458f0fa074c9b1877057e990d8bfe19c4)
                        }
                        case 25{
                            high := xor(high,0xa8e2c0c999fcaada69d2aacadc1c4642f4d223077fe27e8c1366aa071594e637)
                            low := xor(low,0xce1066bfdb9225529930b52aaeaa9a3e31ff7eb45e1f945a150ac49c0ccdac2d)
                        }
                        case 26{
                            high := xor(high,0xd8a8a217b82ea6e5d6a7465967b7e3e6836eef4ab6f900747fa3ea4bcb038123)
                            low := xor(low,0xbf069f551fa83fc4d6ebdb2316f0a13719a7110d5ff3b55ffb633868b466f845)
                        }
                        case 27{
                            high := xor(high,0xbce0c19888404296ddbdd88b7fc5254663a553f8a728405a378a2bce6862e570)
                            low := xor(low,0xefb77e7dc611625e32515c156984b765e84059769ba386fdd4eed4d9f8fe0309)
                        }
                        case 28{
                            high := xor(high,0x0ce54601baf879c2d85240571d8c1d7a72c0a3a95a1ffbde82f33a455143f446)
                            low := xor(low,0x29c7e182e536c32f5a6f245b44272adbcb701d9cf76137ec0841f145e7042ecc)
                        }
                        case 29{
                            high := xor(high,0xf1277dd7745cf92ca8fe65fed3e2d7cf54c513ef6079bc2db66336b0101e383b)
                            low := xor(low,0xbcd7575325be238a56a6f0beeeffcc175ea31f3d0ae772f5f76de3de1bbecdad)
                        }
                        case 30{
                            high := xor(high,0xc9107d43f7e38dce618358cd5c833f04f6975906de4177e567d314dcb4760f3e)
                            low := xor(low,0x56ce58880e8345a8bff6b1bf78dfb112f1709c1e7bb8ed8b902402b9daa64ae0)
                        }
                        case 31{
                            high := xor(high,0x46b71d897eee035fbe37650999648f3a0863ea1f49ad888779bdecc53c10b568)
                            low := xor(low,0x5f2e4bae04ef20ab72f8ce7b521e1ebe145255352e8af95b9094ccfdbcf36713)
                        }
                        case 32{
                            high := xor(high,0xc73953efd4b914746554ec2de3885c9603dc73b7931688a9cbbef1822b77cfc9)
                            low := xor(low,0x632a32bdd2115dcc1ae5533d32684e134cc5a00413321bde62cbd38d78383a3b)
                        }
                        case 33{
                            high := xor(high,0xd00686f19f601ee77eaf23de3110c4929c3512097eb89d526d566eacc2efd226)
                            low := xor(low,0x32e9fac55222727409f84725b8d0b60572291f0271b5c34b3dbfcbb804a02263)
                        }
                        case 34{
                            high := xor(high,0x55ba597fd4e4037dc813e1beffddeefac3c058f387010f2e1dfcf55fc694eeeb)
                            low := xor(low,0xa9c01a7498c2fc6be57e1428dd265a71836b956d7e46ab1a5835d54150b32505)
                        }
                        case 35{
                            high := xor(high,0xe640913cbb486079fe496263113c5b6993cd66205efe823b2d657b40b46dfc6c)
                            low := xor(low,0x57710c69fe9fadebb5f8728ae3224170ca28b751fdabae565ab12c3ca697c457)
                        }
                        case 36{
                            high := xor(high,0xd28fa2b7056579f29fd9d810e3557478d88d89aba72a94226d47abd0405bcbd9)
                            low := xor(low,0x6f83ebaf13caec76fceb9ee22e922df7ce9856dfc05e93222772c854b67f2a32)
                        }
                        case 37{
                            high := xor(high,0x6d1af28d3a78cf77dff411e461c74ca9ed8b842e728808456e857085c6404932)
                            low := xor(low,0xee37f6bc27116f485e9ec45a8ea2a51fa5573db7a746d036486b47685b438f3b)
                        }
                        case 38{
                            high := xor(high,0x18c54a5c64fcf08ee993cdc135c1ead39de07de7321b841c87423c5e071aa0f6)
                            low := xor(low,0x962eb75bbb06bdd2dcdb5363389752f283d9cc88d014adc6c71121bb2372f938)
                        }
                        case 39{
                            high := xor(high,0xcaff265062be895156dccaffac4084c009712e951d3c288f1b085744e1d3cfef)
                            low := xor(low,0x5c9a812e6611fd5985e460441981d8855a4c903f43f30d4b7d1d601bdd3c3391)
                        }
                        case 40{
                            high := xor(high,0x030ec65ec12878cd72e795fed0c76abd1ec085db7cbb61fa93e8dd1e8582eb06)
                            low := xor(low,0x73563144049d4e7e5fd5aefe7b842a0075ced665bb32d4584e83bba78f15151f)
                        }
                        case 41{
                            high := xor(high,0x7795a125f0842455499af99d565cc7faa3b1278d3f27ce7496ca058e8a497443)
                            low := xor(low,0xa6fb8caec115aa2117504923e4932402aea886c28eb79af5ebd5ea6bc7980d3b)
                        }
                        case 42{
                            high := xor(high,0x71369315796e6a663a7ec708b05175c8e02b74e7eb377ad36c8c1f54b980c374)
                            low := xor(low,0x59aee281449cb799e01f5605ed0e085ec9a1a3b4aac481b1c935c39cb7d8ce7f)
                        }

                        default{
                            revert(0x80,0x00)
                        }
                    //Injection of Constants
                    }

                    {//Addition-Rotation-Addition, a.k.a. ARA
                        new_low := byte(0x20, 0x00)
                        new_high := byte(0x20, 0x00)

                    //0,1
                        let state_vector_high := byte(0x20, 0x00)
                        let state_vector_low := byte(0x20, 0x00)
                        let output_high := byte(0x20, 0x00)
                        let output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0xE0,high)

                        state_vector_low := shr(0xC0,and(high,0x00000000ffffffff000000000000000000000000000000000000000000000000))

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_high := or(new_high,shl(0xE0, output_high))
                        new_high := or(new_high,shl(0xC0, output_low))

                    //2,3
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0xA0,and(high,0x0000000000000000ffffffff0000000000000000000000000000000000000000))

                        state_vector_low := shr(0x80,and(high,0x000000000000000000000000ffffffff00000000000000000000000000000000))

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_high := or(new_high,shl(0xA0, output_high))
                        new_high := or(new_high,shl(0x80, output_low))

                    //4,5
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0x60,and(high,0x00000000000000000000000000000000ffffffff000000000000000000000000))

                        state_vector_low := shr(0x40,and(high,0x0000000000000000000000000000000000000000ffffffff0000000000000000))

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_high := or(new_high,shl(0x60, output_high))
                        new_high := or(new_high,shl(0x40, output_low))


                    //6,7
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0x20,and(high,0x000000000000000000000000000000000000000000000000ffffffff00000000))

                        state_vector_low := and(high,0x00000000000000000000000000000000000000000000000000000000ffffffff)

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_high := or(new_high,shl(0x20, output_high))
                        new_high := or(new_high,output_low)

                    //8,9
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0xE0,low)

                        state_vector_low := shr(0xC0,and(low,0x00000000ffffffff000000000000000000000000000000000000000000000000))

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_low := or(new_low,shl(0xE0, output_high))
                        new_low := or(new_low,shl(0xC0, output_low))

                    //a,b
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0xA0,and(low,0x0000000000000000ffffffff0000000000000000000000000000000000000000))

                        state_vector_low := shr(0x80,and(low,0x000000000000000000000000ffffffff00000000000000000000000000000000))

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_low := or(new_low,shl(0xA0, output_high))
                        new_low := or(new_low,shl(0x80, output_low))

                    //c,d
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0x60,and(low,0x00000000000000000000000000000000ffffffff000000000000000000000000))

                        state_vector_low := shr(0x40,and(low,0x0000000000000000000000000000000000000000ffffffff0000000000000000))

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_low := or(new_low,shl(0x60, output_high))
                        new_low := or(new_low,shl(0x40, output_low))


                    //e,f
                        state_vector_high := byte(0x20, 0x00)
                        state_vector_low := byte(0x20, 0x00)
                        output_high := byte(0x20, 0x00)
                        output_low := byte(0x20, 0x00)

                    // output_vector[2*i] = state_vector[2*i] + state_vector[2*i + 1]

                        state_vector_high := shr(0x20,and(low,0x000000000000000000000000000000000000000000000000ffffffff00000000))

                        state_vector_low := and(low,0x00000000000000000000000000000000000000000000000000000000ffffffff)

                        output_high := and(
                        add(state_vector_high,state_vector_low),
                        0x00000000000000000000000000000000000000000000000000000000ffffffff)

                    // output_vector[2*i] = output_vector[2*i] <<< 8

                        output_high := or(
                        and(
                        shl(8,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(24,output_high)
                        )

                    // output_vector[2*i + 1] = state_vector[2*i + 1] <<< 24
                        output_low := or(
                        and(
                        shl(24,state_vector_low),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        ),
                        shr(8,state_vector_low)
                        )

                    // output_vector[2*i + 1] = output_vector[2*i + 1] + output_vector[2*i]
                        output_low := and(
                        add(output_low,output_high),0x00000000000000000000000000000000000000000000000000000000ffffffff
                        )

                    // save output back to state
                        new_low := or(new_low,shl(0x20, output_high))
                        new_low := or(new_low,output_low)

                        high := new_high
                        low := new_low
                    }
                // round 43 :)
                }
            }

        //=================================================SQUEEZE===========================================================
            {
            // due to r =256 bit and OUTPUT_LEN = 256 bit, we doesn't need F()

            // reverse 'high' in chunk of every 32 bits
            // e.g. 0xAABBCCDD EEFF0011 ->
            //      0xDDCCBBAA 1100FFEE
            // here use drop 'low' and re-use it as container
                low := byte(0x20, 0x00)

                low := or(low,shr(24,and(high,0xff00000000000000000000000000000000000000000000000000000000000000)))
                low := or(low,shr(8,and(high,0x00ff000000000000000000000000000000000000000000000000000000000000)))
                low := or(low,shl(8,and(high,0x0000ff0000000000000000000000000000000000000000000000000000000000)))
                low := or(low,shl(24,and(high,0x000000ff00000000000000000000000000000000000000000000000000000000)))

                low := or(low,shr(24,and(high,0x00000000ff000000000000000000000000000000000000000000000000000000)))
                low := or(low,shr(8,and(high,0x0000000000ff0000000000000000000000000000000000000000000000000000)))
                low := or(low,shl(8,and(high,0x000000000000ff00000000000000000000000000000000000000000000000000)))
                low := or(low,shl(24,and(high,0x00000000000000ff000000000000000000000000000000000000000000000000)))

                low := or(low,shr(24,and(high,0x0000000000000000ff0000000000000000000000000000000000000000000000)))
                low := or(low,shr(8,and(high,0x000000000000000000ff00000000000000000000000000000000000000000000)))
                low := or(low,shl(8,and(high,0x00000000000000000000ff000000000000000000000000000000000000000000)))
                low := or(low,shl(24,and(high,0x0000000000000000000000ff0000000000000000000000000000000000000000)))

                low := or(low,shr(24,and(high,0x000000000000000000000000ff00000000000000000000000000000000000000)))
                low := or(low,shr(8,and(high,0x00000000000000000000000000ff000000000000000000000000000000000000)))
                low := or(low,shl(8,and(high,0x0000000000000000000000000000ff0000000000000000000000000000000000)))
                low := or(low,shl(24,and(high,0x000000000000000000000000000000ff00000000000000000000000000000000)))

                low := or(low,shr(24,and(high,0x00000000000000000000000000000000ff000000000000000000000000000000)))
                low := or(low,shr(8,and(high,0x0000000000000000000000000000000000ff0000000000000000000000000000)))
                low := or(low,shl(8,and(high,0x000000000000000000000000000000000000ff00000000000000000000000000)))
                low := or(low,shl(24,and(high,0x00000000000000000000000000000000000000ff000000000000000000000000)))

                low := or(low,shr(24,and(high,0x0000000000000000000000000000000000000000ff0000000000000000000000)))
                low := or(low,shr(8,and(high,0x000000000000000000000000000000000000000000ff00000000000000000000)))
                low := or(low,shl(8,and(high,0x00000000000000000000000000000000000000000000ff000000000000000000)))
                low := or(low,shl(24,and(high,0x0000000000000000000000000000000000000000000000ff0000000000000000)))

                low := or(low,shr(24,and(high,0x000000000000000000000000000000000000000000000000ff00000000000000)))
                low := or(low,shr(8,and(high,0x00000000000000000000000000000000000000000000000000ff000000000000)))
                low := or(low,shl(8,and(high,0x0000000000000000000000000000000000000000000000000000ff0000000000)))
                low := or(low,shl(24,and(high,0x000000000000000000000000000000000000000000000000000000ff00000000)))

                low := or(low,shr(24,and(high,0x00000000000000000000000000000000000000000000000000000000ff000000)))
                low := or(low,shr(8,and(high,0x0000000000000000000000000000000000000000000000000000000000ff0000)))
                low := or(low,shl(8,and(high,0x000000000000000000000000000000000000000000000000000000000000ff00)))
                low := or(low,shl(24,and(high,0x00000000000000000000000000000000000000000000000000000000000000ff)))

            // the memory is still frozen in 0x80:0xA0 and we just re-use it, saving gas of clean and update

                mstore(0x80,low)
                return(0x80,0x20)
            }
        // assembly
        }


    }
}
