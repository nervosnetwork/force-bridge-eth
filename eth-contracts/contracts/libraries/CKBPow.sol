// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "../libraries/TypedMemView.sol";
import {CKBCrypto} from "../libraries/CKBCrypto.sol";
import {SafeMath} from "../libraries/SafeMath.sol";
import {ViewCKB} from "../libraries/ViewCKB.sol";
import {ViewSpv} from "../libraries/ViewSpv.sol";
import {EaglesongLib} from "../libraries/EaglesongLib.sol";
import {ICKBChain} from "../interfaces/ICKBChain.sol";
import {ICKBSpv} from "../interfaces/ICKBSpv.sol";
//import "hardhat/console.sol";

// @dev    reference code:  https://github.com/nervosnetwork/ckb/blob/master/util/types/src/utilities/difficulty.rs
library CKBPow {
    uint256 constant public MAX_UIN256 = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

    function compactToTarget(uint32 compact) internal pure returns (uint256, bool) {
        /*
            let exponent = compact >> 24;
            let mut mantissa = U256::from(compact & 0x00ff_ffff);

            let mut ret;
            if exponent <= 3 {
                mantissa >>= 8 * (3 - exponent);
                ret = mantissa.clone();
            } else {
                ret = mantissa.clone();
                ret <<= 8 * (exponent - 3);
            }

            let overflow = !mantissa.is_zero() && (exponent > 32);
            (ret, overflow)
        */

        uint32 exponent = compact >> 24;
        uint256 mantissa = uint256(compact & uint32(0x00ffffff));
        uint256 ret;
        if (exponent <= 3) {
            mantissa >>= 8 * (3 - exponent);
            ret = mantissa;
        } else {
            ret = mantissa;
            ret <<= 8 * (exponent - 3);
        }
        bool overflow = mantissa != 0 && (exponent > 32);
        return (ret, overflow);
    }

    function targetToDifficulty(uint256 target) internal pure returns (uint256) {
        /*
            const ONE: U256 = U256::one();
            // ONE << 256
            const HSPACE: U512 = u512!("0x10000000000000000000000000000000000000000000000000000000000000000");

            fn target_to_difficulty(target: &U256) -> U256 {
                if target == &ONE {
                    U256::max_value()
                } else {
                    let (target, _): (U512, bool) = target.convert_into();
                    (HSPACE / target).convert_into().0
                }
            }
        */

        if (target == 1) {
            return MAX_UIN256;
        }

        if (MAX_UIN256 % target == target - 1) {
            return MAX_UIN256 / target + 1;
        }

        return MAX_UIN256 / target;
    }
}
