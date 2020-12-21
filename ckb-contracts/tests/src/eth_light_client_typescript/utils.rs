use super::types::*;
use ckb_tool::ckb_types::{packed::*, prelude::*};
use force_eth_types::generated::{basic, eth_header_cell};
use molecule::bytes::Bytes;
use molecule::prelude::*;
use std::convert::From;

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
