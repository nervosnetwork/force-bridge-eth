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
    // ADD_HEADERS_TYPEHASH = keccak256("AddHeaders(bytes data)");
    bytes32 public constant ADD_HEADERS_TYPEHASH = 0xfa98e6fcbad03c89f421602d77ef593b53ee59d7442ea61663cb69d2a29a764d;


    // CHAIN_VERSION means chain_id on CKB CHAIN
    uint32 public constant CHAIN_VERSION = 0;

    //==============================================================================

    // We store the hashes of the blocks for the past `CanonicalGcThreshold` headers.
    // Events that happen past this threshold cannot be verified by the client.
    // It is desirable that this number is larger than 7 days worth of headers, which is roughly
    // 40k ckb blocks. So this number should be 40k in production.
    uint64 public CanonicalGcThreshold;
    // We store full information about the headers for the past `FinalizedGcThreshold` blocks.
    // This is required to be able to adjust the canonical chain when the fork switch happens.
    // The commonly used number is 500 blocks, so this number should be 500 in production.
    uint64 public FinalizedGcThreshold;

    // Whether the contract was initialized.
    bool public initialized;
    uint64 public latestBlockNumber;
    uint64 public initBlockNumber;
    CKBChainV2Library.BlockHeader latestHeader;

    // Todo will remove the governance when optimistic phase
    address public governance;
    uint256 MOCK_DIFFICULTY = 1;

    // Hashes of the canonical chain mapped to their numbers. Stores up to `canonical_gc_threshold`
    // entries.
    // header number -> header hash
    mapping(uint64 => bytes32) canonicalHeaderHashes;

    // TransactionRoots of the canonical chain mapped to their headerHash. Stores up to `canonical_gc_threshold`
    // entries.
    // header hash -> transactionRoots from the header
    mapping(bytes32 => bytes32) canonicalTransactionsRoots;

    // All known header hashes. Stores up to `finalized_gc_threshold`.
    // header number -> hashes of all headers with this number.
    mapping(uint64 => bytes32[]) allHeaderHashes;

    // Known headers. Stores up to `finalized_gc_threshold`.
    mapping(bytes32 => CKBChainV2Library.BlockHeader) blockHeaders;


    bytes32 public DOMAIN_SEPARATOR;
    // if the number of verified signatures has reached `multisigThreshold_`, validators approve the tx
    uint public multisigThreshold_;
    address[] validators_;

}
