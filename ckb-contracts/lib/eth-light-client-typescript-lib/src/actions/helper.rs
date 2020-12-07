#![allow(dead_code)]
#![allow(clippy::all)]

use crate::adapter::Adapter;
use crate::debug;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use eth_spv_lib::eth_types::*;

use force_eth_types::{
    eth_header_cell::ETHHeaderCellDataView,
    generated::eth_header_cell::{
        DagsMerkleRootsReader, DoubleNodeWithMerkleProofReader, ETHLightClientWitnessReader,
        MerkleProofVecReader,
    },
};
use molecule::prelude::Reader;

#[derive(Default, Debug, Clone)]
pub struct DoubleNodeWithMerkleProof {
    pub dag_nodes: Vec<H512>, // [H512; 2]
    pub proof: Vec<H128>,
}

impl DoubleNodeWithMerkleProof {
    pub fn new(f: Vec<H512>, s: Vec<H128>) -> Self {
        Self {
            dag_nodes: f,
            proof: s,
        }
    }
    fn truncate_to_h128(arr: H256) -> H128 {
        let mut data = [0u8; 16];
        data.copy_from_slice(&(arr.0).0[16..]);
        H128(data.into())
    }

    fn hash_h128(l: H128, r: H128) -> H128 {
        let mut data = [0u8; 64];
        data[16..32].copy_from_slice(&(l.0).0);
        data[48..64].copy_from_slice(&(r.0).0);
        Self::truncate_to_h128(hash256(&data).into())
    }

    pub fn apply_merkle_proof(&self, index: u64) -> H128 {
        let mut data = [0u8; 128];
        data[..64].copy_from_slice(&(self.dag_nodes[0].0).0);
        data[64..].copy_from_slice(&(self.dag_nodes[1].0).0);

        let mut leaf = Self::truncate_to_h128(hash256(&data).into());

        for i in 0..self.proof.len() {
            if (index >> i as u64) % 2 == 0 {
                leaf = Self::hash_h128(leaf, self.proof[i]);
            } else {
                leaf = Self::hash_h128(self.proof[i], leaf);
            }
        }
        leaf
    }
}

fn verify_merkle_proof<T: Adapter>(
    data_loader: T,
    witness: ETHLightClientWitnessReader,
    output: &ETHHeaderCellDataView,
    header: &BlockHeader,
    prev: Option<BlockHeader>,
) {
    if MerkleProofVecReader::verify(&output.merkle_proofs, false).is_err() {
        panic!("verify_merkle_proof, invalid output.merkle_proof");
    }
    let merkle_proof_vec = MerkleProofVecReader::new_unchecked(&output.merkle_proofs);
    let mut proofs = vec![];
    let merkle_proofs = merkle_proof_vec.get_unchecked(merkle_proof_vec.len() - 1);
    for i in 0..merkle_proofs.len() {
        let proof_raw = merkle_proofs.get_unchecked(i).raw_data();
        let proof = parse_proof(proof_raw);
        proofs.push(proof);
    }

    // parse dep data
    let merkle_root = parse_dep_data(data_loader, witness, header.number);

    if !verify_header(&header, prev, merkle_root, &proofs) {
        panic!("verify_witness, verify header fail");
    }
}

pub fn verify_header(
    header: &BlockHeader,
    prev: Option<BlockHeader>,
    merkle_root: H128,
    dag_nodes: &[DoubleNodeWithMerkleProof],
) -> bool {
    let (_mix_hash, result) = hashimoto_merkle(
        &header.partial_hash.unwrap(),
        &header.nonce,
        header.number,
        merkle_root,
        dag_nodes,
    );
    debug!("verify_header header: {:?}", header);
    // See YellowPaper formula (50) in section 4.3.4
    // 1. Simplified difficulty check to conform adjusting difficulty bomb
    // 2. Added condition: header.parent_hash() == prev.hash()
    let result = U256((result.0).0.into()) < U256(ethash::cross_boundary(header.difficulty.0))
        && (header.difficulty < header.difficulty * 101 / 100
            && header.difficulty > header.difficulty * 99 / 100)
        && header.gas_used <= header.gas_limit
        && header.gas_limit >= U256(5000.into())
        && header.extra_data.len() <= 32;
    match prev {
        Some(prev) => {
            debug!("verify_header prev: {:?}", prev);
            result
                && header.gas_limit < prev.gas_limit * 1025 / 1024
                && header.gas_limit > prev.gas_limit * 1023 / 1024
                && header.timestamp > prev.timestamp
                && header.number == prev.number + 1
                && header.parent_hash == prev.hash.unwrap()
        }
        None => result,
    }
}

/// Verify merkle paths to the DAG nodes.
fn hashimoto_merkle(
    header_hash: &H256,
    nonce: &H64,
    header_number: u64,
    merkle_root: H128,
    nodes: &[DoubleNodeWithMerkleProof],
) -> (H256, H256) {
    let mut index = 0;
    let pair = ethash::hashimoto_with_hasher(
        header_hash.0,
        nonce.0,
        ethash::get_full_size(header_number as usize / 30000),
        |offset| {
            let idx = index;
            debug!("hashimoto_with_hasher index: {}", index);
            index += 1;
            // Each two nodes are packed into single 128 bytes with Merkle proof
            let node = &nodes[idx / 2];
            if idx % 2 == 0 {
                // Divide by 2 to adjust offset for 64-byte words instead of 128-byte
                assert_eq!(merkle_root, node.apply_merkle_proof((offset / 2) as u64));
            };

            // Reverse each 32 bytes for ETHASH compatibility
            let mut data = (node.dag_nodes[idx % 2].0).0;
            data[..32].reverse();
            data[32..].reverse();
            data.into()
        },
        keccak256,
        keccak512,
    );

    (H256(pair.0), H256(pair.1))
}

fn parse_proof(proof_raw: &[u8]) -> DoubleNodeWithMerkleProof {
    if DoubleNodeWithMerkleProofReader::verify(&proof_raw, false).is_err() {
        panic!("invalid proof raw");
    }
    let merkle_proof = DoubleNodeWithMerkleProofReader::new_unchecked(proof_raw);
    let mut dag_nodes = vec![];
    for i in 0..merkle_proof.dag_nodes().len() {
        let mut node = [0u8; 64];
        node.copy_from_slice(merkle_proof.dag_nodes().get_unchecked(i).raw_data());
        dag_nodes.push(H512(node.into()));
    }
    let mut proofs = vec![];
    for i in 0..merkle_proof.proof().len() {
        let mut proof = [0u8; 16];
        proof.copy_from_slice(merkle_proof.proof().get_unchecked(i).raw_data());
        proofs.push(H128(proof.into()));
    }
    DoubleNodeWithMerkleProof::new(dag_nodes, proofs)
}

fn parse_dep_data<T: Adapter>(
    data_loader: T,
    witness: ETHLightClientWitnessReader,
    number: u64,
) -> H128 {
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();
    if cell_dep_index_list.len() != 1 {
        panic!("parse_dep_data, witness cell dep index len is not 1");
    }
    let dep_data = data_loader.load_data_from_dep(cell_dep_index_list[0].into());
    // debug!("dep data is {:?}", &dep_data);
    if DagsMerkleRootsReader::verify(&dep_data, false).is_err() {
        panic!(
            "parse_dep_data, invalid dags {:?} {:?}",
            dep_data, cell_dep_index_list[0]
        );
    }
    let dags_reader = DagsMerkleRootsReader::new_unchecked(&dep_data);
    let idx: usize = (number / 30000) as usize;
    let merkle_root_tmp = dags_reader
        .dags_merkle_roots()
        .get_unchecked(idx)
        .raw_data();
    let mut merkle_root = [0u8; 16];
    merkle_root.copy_from_slice(merkle_root_tmp);
    H128(merkle_root.into())
}
