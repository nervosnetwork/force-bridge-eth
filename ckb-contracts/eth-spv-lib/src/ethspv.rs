extern crate alloc;
use crate::eth_types::*;
use rlp::Rlp;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

/// verify the log entry is valid.
pub fn verify_log_entry(
    log_index: u64,
    log_entry_data: Vec<u8>,
    receipt_index: u64,
    receipt_data: Vec<u8>,
    receipts_root: H256,
    proof: Vec<Vec<u8>>,
) -> bool {
    let log_entry: LogEntry = rlp::decode(log_entry_data.as_slice()).unwrap();
    let receipt: Receipt = rlp::decode(receipt_data.as_slice()).unwrap();
    // Verify log_entry included in receipt.
    assert_eq!(receipt.logs[log_index as usize], log_entry);
    // Verify the trie proof is valid.
    verify_trie_proof(
        receipts_root,
        rlp::encode(&receipt_index),
        proof,
        receipt_data,
    )
}

/// Iterate the proof following the key.
/// Return True if the value at the leaf is equal to the expected value.
/// @param expected_root is the expected root of the current proof node.
/// @param key is the key for which we are proving the value.
/// @param proof is the proof the key nibbles as path.
/// @param expected_value is the key's value expected to be stored in
///     the last node (leaf node) of the proof.
///
/// Patricia Trie: https://github.com/ethereum/wiki/wiki/Patricia-Tree#example-trie
/// Patricia Img:  https://ethereum.stackexchange.com/questions/268/ethereum-block-architecture/6413#6413
///
/// Verification:  https://github.com/slockit/in3/wiki/Ethereum-Verification-and-MerkleProof#receipt-proof
/// Article:       https://medium.com/@ouvrard.pierre.alain/merkle-proof-verification-for-ethereum-patricia-tree-48f29658eec
/// Python impl:   https://gist.github.com/paouvrard/7bb947bf5de0fa0dc69d0d254d82252a
/// JS impl:       https://github.com/slockit/in3/blob/master/src/util/merkleProof.ts
///
fn verify_trie_proof(
    expected_root: H256,
    key: Vec<u8>,
    proof: Vec<Vec<u8>>,
    expected_value: Vec<u8>,
) -> bool {
    let mut actual_key = vec![];
    for el in key {
        if actual_key.len() + 1 == proof.len() {
            actual_key.push(el);
        } else {
            actual_key.push(el / 16);
            actual_key.push(el % 16);
        }
    }
    _verify_trie_proof(expected_root, actual_key, proof, 0, 0, expected_value)
}

fn _verify_trie_proof(
    expected_root: H256,
    key: Vec<u8>,
    proof: Vec<Vec<u8>>,
    key_index: usize,
    proof_index: usize,
    expected_value: Vec<u8>,
) -> bool {
    let node = &proof[proof_index];
    let dec = Rlp::new(&node.as_slice());

    if key_index == 0 {
        // trie root is always a hash
        assert_eq!(keccak256(node), (expected_root.0).0);
    } else if node.len() < 32 {
        // if rlp < 32 bytes, then it is not hashed
        assert_eq!(dec.as_raw(), (expected_root.0).0);
    } else {
        assert_eq!(keccak256(node), (expected_root.0).0);
    }

    if dec.iter().count() == 17 {
        // branch node
        if key_index == key.len() {
            if dec
                .at(dec.iter().count() - 1)
                .unwrap()
                .as_val::<Vec<u8>>()
                .unwrap()
                == expected_value
            {
                // value stored in the branch
                return true;
            }
        } else if key_index < key.len() {
            let new_expected_root = dec
                .at(key[key_index] as usize)
                .unwrap()
                .as_val::<Vec<u8>>()
                .unwrap();
            if new_expected_root.len() != 0 {
                return _verify_trie_proof(
                    new_expected_root.into(),
                    key,
                    proof,
                    key_index + 1,
                    proof_index + 1,
                    expected_value,
                );
            }
        } else {
            panic!("This should not be reached if the proof has the correct format");
        }
    } else if dec.iter().count() == 2 {
        // leaf or extension node
        // get prefix and optional nibble from the first byte
        let nibbles = extract_nibbles(dec.at(0).unwrap().as_val::<Vec<u8>>().unwrap());
        let (prefix, nibble) = (nibbles[0], nibbles[1]);

        if prefix == 2 {
            // even leaf node
            let key_end = &nibbles[2..];
            if concat_nibbles(key_end.to_vec()) == &key[key_index..]
                && expected_value == dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap()
            {
                return true;
            }
        } else if prefix == 3 {
            // odd leaf node
            let key_end = &nibbles[2..];
            if nibble == key[key_index]
                && concat_nibbles(key_end.to_vec()) == &key[key_index + 1..]
                && expected_value == dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap()
            {
                return true;
            }
        } else if prefix == 0 {
            // even extension node
            let shared_nibbles = &nibbles[2..];
            let extension_length = shared_nibbles.len();
            if concat_nibbles(shared_nibbles.to_vec())
                == &key[key_index..key_index + extension_length]
            {
                let new_expected_root = dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap();
                return _verify_trie_proof(
                    new_expected_root.into(),
                    key,
                    proof,
                    key_index + extension_length,
                    proof_index + 1,
                    expected_value,
                );
            }
        } else if prefix == 1 {
            // odd extension node
            let shared_nibbles = &nibbles[2..];
            let extension_length = 1 + shared_nibbles.len();
            if nibble == key[key_index]
                && concat_nibbles(shared_nibbles.to_vec())
                    == &key[key_index + 1..key_index + extension_length]
            {
                let new_expected_root = dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap();
                return _verify_trie_proof(
                    new_expected_root.into(),
                    key,
                    proof,
                    key_index + extension_length,
                    proof_index + 1,
                    expected_value,
                );
            }
        } else {
            panic!("This should not be reached if the proof has the correct format");
        }
    } else {
        panic!("This should not be reached if the proof has the correct format");
    }

    expected_value.len() == 0
}

fn extract_nibbles(a: Vec<u8>) -> Vec<u8> {
    a.iter().flat_map(|b| vec![b >> 4, b & 0x0F]).collect()
}

fn concat_nibbles(a: Vec<u8>) -> Vec<u8> {
    a.iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .zip(a.iter().enumerate().filter(|(i, _)| i % 2 == 1))
        .map(|((_, x), (_, y))| (x << 4) | y)
        .collect()
}
