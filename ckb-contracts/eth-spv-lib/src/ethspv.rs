extern crate alloc;
use crate::eth_types::*;
use rlp::Rlp;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

/// verify the log entry is valid.
pub fn verify_log_entry(
    receipt_index: u64,
    receipt_data: Vec<u8>,
    receipts_root: H256,
    proof: Vec<Vec<u8>>,
) -> Receipt {
    let receipt: Receipt = rlp::decode(receipt_data.as_slice()).expect("invalid receipt data");
    // Verify the trie proof is valid.
    assert!(
        verify_trie_proof(
            receipts_root,
            rlp::encode(&receipt_index),
            proof,
            receipt_data,
        ),
        "receipt proof is invalid"
    );
    receipt
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

#[allow(clippy::all)]
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
    match dec.iter().count() {
        17 => {
            // branch node
            match key.len() {
                len if len == key_index => {
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
                }
                len if len > key_index => {
                    let new_expected_root = dec
                        .at(key[key_index] as usize)
                        .unwrap()
                        .as_val::<Vec<u8>>()
                        .unwrap();
                    if !new_expected_root.is_empty() {
                        return _verify_trie_proof(
                            new_expected_root.into(),
                            key,
                            proof,
                            key_index + 1,
                            proof_index + 1,
                            expected_value,
                        );
                    }
                }
                _ => {
                    panic!("This should not be reached if the proof has the correct format");
                }
            }
        }
        2 => {
            // leaf or extension node
            // get prefix and optional nibble from the first byte
            let nibbles = extract_nibbles(dec.at(0).unwrap().as_val::<Vec<u8>>().unwrap());
            let (prefix, nibble) = (nibbles[0], nibbles[1]);

            match prefix {
                2 => {
                    // even leaf node
                    let key_end = &nibbles[2..];
                    if concat_nibbles(key_end.to_vec()) == &key[key_index..]
                        && expected_value == dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap()
                    {
                        return true;
                    }
                }
                3 => {
                    // odd leaf node
                    let key_end = &nibbles[2..];
                    if nibble == key[key_index]
                        && concat_nibbles(key_end.to_vec()) == &key[key_index + 1..]
                        && expected_value == dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap()
                    {
                        return true;
                    }
                }
                0 => {
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
                }
                1 => {
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
                }
                _ => {
                    panic!("This should not be reached if the proof has the correct format");
                }
            }
        }
        _ => {
            panic!("This should not be reached if the proof has the correct format");
        }
    }
    expected_value.is_empty()
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
