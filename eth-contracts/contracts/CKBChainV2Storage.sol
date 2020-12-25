// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import {TypedMemView} from "./libraries/TypedMemView.sol";
import {CKBCrypto} from "./libraries/CKBCrypto.sol";
import {Blake2b} from "./libraries/Blake2b.sol";
import {SafeMath} from "./libraries/SafeMath.sol";
import {ViewCKB} from "./libraries/ViewCKB.sol";
import {ViewSpv} from "./libraries/ViewSpv.sol";
import {CKBPow} from "./libraries/CKBPow.sol";
import {EaglesongLib} from "./libraries/EaglesongLib.sol";
import {ICKBChainV2} from "./interfaces/ICKBChainV2.sol";
import {ICKBSpv} from "./interfaces/ICKBSpv.sol";
import {MultisigUtils} from "./libraries/MultisigUtils.sol";
import "./proxy/Proxy.sol";
import "./CKBChainV2Layout.sol";

// tools below just for test, they will be removed before production ready
//import "hardhat/console.sol";

contract CKBChainV2Storage is Proxy, CKBChainV2Layout {
    constructor(uint64 canonicalGcThreshold, address _proxy_admin) Proxy(_proxy_admin) {
        governance = msg.sender;

        // set init threshold
        CanonicalGcThreshold = canonicalGcThreshold;
    }
}
