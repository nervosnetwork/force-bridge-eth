// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
pragma abicoder v2;

import "./CKBChainV2Library.sol";


contract CKBChainV2Layout {
    //constants take no space, declare them at the begin
    //==============================================================================

    // refer to https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md
    uint public constant SIGNATURE_SIZE = 65;
    uint public constant VALIDATORS_SIZE_LIMIT = 20;
    string public constant NAME_712 = "Force Bridge CKBChain";
    // ADD_HEADERS_TYPEHASH = keccak256("AddHeaders(bytes[] tinyHeaders)");
    bytes32 public constant ADD_HEADERS_TYPEHASH = 0x1dac851def8ec317cf44b4a6cf63dabe82895259e6290d4c2ef271700bfce584;

    //==============================================================================

    // We store the hashes of the blocks for the past `CanonicalGcThreshold` headers.
    // Events that happen past this threshold cannot be verified by the client.
    // It is desirable that this number is larger than 7 days worth of headers, which is roughly
    // 40k ckb blocks. So this number should be 40k in production.
    uint64 public CanonicalGcThreshold;

    uint64 public latestBlockNumber;
    uint64 public initBlockNumber;

    // Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    // entries.
    // header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    // TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    // entries.
    // header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionsRoots;


    bytes32 public DOMAIN_SEPARATOR;
    // if the number of verified signatures has reached `multisigThreshold_`, validators approve the tx
    uint public multisigThreshold_;
    address[] validators_;
}
