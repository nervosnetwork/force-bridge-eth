use super::types::*;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use eth_spv_lib::eth_types::*;
use force_eth_types::generated::{basic, eth_header_cell};
use molecule::bytes::Bytes;
use molecule::prelude::*;
use std::convert::From;

pub fn create_hash_data(block_with_proof: &BlockWithProofs) -> basic::Bytes {
    let header: BlockHeader = rlp::decode(block_with_proof.header_rlp.0.as_slice()).unwrap();
    header.hash.unwrap().0.as_bytes().to_vec().into()
}

pub fn create_data(
    block_with_proof: &BlockWithProofs,
    pre_block_difficulty: u64,
) -> (basic::Bytes, u64) {
    let header: BlockHeader = rlp::decode(block_with_proof.header_rlp.0.as_slice()).unwrap();
    let header_info = eth_header_cell::ETHHeaderInfo::new_builder()
        .header(basic::Bytes::from(block_with_proof.header_rlp.0.clone()))
        .total_difficulty(
            header
                .difficulty
                .0
                .as_u64()
                .checked_add(pre_block_difficulty)
                .unwrap()
                .into(),
        )
        .hash(basic::Byte32::from_slice(header.hash.unwrap().0.as_bytes()).unwrap())
        .build();
    (
        header_info.as_slice().to_vec().into(),
        header
            .difficulty
            .0
            .as_u64()
            .checked_add(pre_block_difficulty)
            .unwrap(),
    )
}

pub fn create_cell_data(
    main: Vec<basic::Bytes>,
    uncle: Option<Vec<basic::Bytes>>,
    blocks: &[BlockWithProofs],
) -> eth_header_cell::ETHHeaderCellData {
    let mut merkle_proofs = vec![];
    for block in blocks {
        let merkle_proof = create_merkle_proof(&block);
        merkle_proofs.push(merkle_proof);
    }
    let merkle_proofs = eth_header_cell::MerkleProofVec::new_builder()
        .set(merkle_proofs)
        .build();

    match uncle {
        Some(u) => eth_header_cell::ETHHeaderCellData::new_builder()
            .headers(
                eth_header_cell::ETHChain::new_builder()
                    .main(basic::BytesVec::new_builder().set(main).build())
                    .uncle(basic::BytesVec::new_builder().set(u).build())
                    .build(),
            )
            .merkle_proofs(merkle_proofs)
            .build(),
        None => eth_header_cell::ETHHeaderCellData::new_builder()
            .headers(
                eth_header_cell::ETHChain::new_builder()
                    .main(basic::BytesVec::new_builder().set(main).build())
                    .build(),
            )
            .merkle_proofs(merkle_proofs)
            .build(),
    }
}

fn create_merkle_proof(block_with_proof: &BlockWithProofs) -> eth_header_cell::MerkleProof {
    let proof_vec = block_with_proof.to_double_node_with_merkle_proof_vec();
    let mut proof_json_vec = vec![];
    for proof in proof_vec {
        let dag_nodes = &proof.dag_nodes;
        let mut dag_nodes_string = vec![];
        for dag in dag_nodes {
            dag_nodes_string.push(hex::encode(dag.0));
        }
        let proof = &proof.proof;
        let mut proof_string = vec![];
        for p in proof {
            proof_string.push(hex::encode(p.0));
        }
        proof_json_vec.push(DoubleNodeWithMerkleProofJson {
            dag_nodes: dag_nodes_string,
            proof: proof_string,
        })
    }

    let mut merkle_proofs: Vec<eth_header_cell::DoubleNodeWithMerkleProof> = vec![];
    for proof_json in proof_json_vec {
        let p: eth_header_cell::DoubleNodeWithMerkleProof = (proof_json).clone().into();
        merkle_proofs.push(p);
    }

    let mut proofs = vec![];
    for merkle_proof in merkle_proofs {
        proofs.push(basic::Bytes::from(merkle_proof.as_slice().to_vec()));
    }

    eth_header_cell::MerkleProof::new_builder()
        .set(proofs)
        .build()
}

pub fn create_witness(header_rlps: Vec<Hex>, cell_dep_index_list: Vec<u8>) -> Bytes {
    let mut headers = vec![];
    for rlp in header_rlps {
        headers.push(basic::Bytes::from(rlp.0.to_vec()))
    }

    let witness_data = eth_header_cell::ETHLightClientWitness::new_builder()
        .headers(basic::BytesVec::new_builder().set(headers).build())
        .cell_dep_index_list(cell_dep_index_list.into())
        .build();
    WitnessArgs::new_builder()
        .input_type(Some(witness_data.as_bytes()).pack())
        .build()
        .as_bytes()
}

pub fn create_dep_data() -> Bytes {
    let dep_data_raw = read_roots_collection_raw();
    let mut dag_root = vec![];
    for i in 0..dep_data_raw.dag_merkle_roots.len() {
        dag_root.push(hex::encode(&dep_data_raw.dag_merkle_roots[i].0).clone());
    }
    let dep_data_string = RootsCollectionJson {
        dag_merkle_roots: dag_root,
    };
    let dep_data: eth_header_cell::DagsMerkleRoots = dep_data_string.into();
    dep_data.as_bytes()
}
