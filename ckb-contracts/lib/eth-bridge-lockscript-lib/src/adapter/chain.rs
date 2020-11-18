use super::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, packed::Script};
use ckb_std::error::SysError;
use ckb_std::high_level::{
    load_cell, load_cell_data, load_cell_lock_hash, load_cell_type, load_input_out_point,
    load_script_hash, load_witness_args, QueryIter,
};
use molecule::prelude::Entity;
use std::prelude::v1::*;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_input_data(&self) -> Vec<u8> {
        load_input_data()
    }

    fn load_script_hash(&self) -> [u8; 32] {
        load_script_hash().unwrap()
    }

    fn load_input_witness_args(&self) -> Result<Bytes, SysError> {
        let witness_args = load_witness_args(0, Source::GroupInput)
            .expect("no witness provided")
            .lock()
            .to_opt()
            .expect("proof witness lock field is none");
        Ok(witness_args.raw_data())
    }

    fn load_cell_dep_data(&self, index: usize) -> Result<Vec<u8>, SysError> {
        load_cell_data(index, Source::CellDep)
    }

    fn lock_hash_exists_in_inputs(&self, data: &[u8]) -> bool {
        QueryIter::new(load_cell_lock_hash, Source::Input).any(|hash| hash.as_ref() == data)
    }

    fn typescript_exists_in_outputs(&self, data: &[u8]) -> bool {
        QueryIter::new(load_cell_type, Source::Output)
            .filter_map(|script_opt| script_opt)
            .any(|script| script.as_slice() == data)
    }

    fn outpoint_exists_in_inputs(&self, data: &[u8]) -> bool {
        QueryIter::new(load_input_out_point, Source::Input)
            .any(|outpoint| outpoint.as_slice() == data)
    }

    fn load_cell_type_lock_data(
        &self,
        index: usize,
        source: Source,
    ) -> Result<(Option<Script>, Script, Vec<u8>), SysError> {
        let cell = load_cell(index, source)?;
        let data = load_cell_data(index, source)?;
        Ok((cell.type_().to_opt(), cell.lock(), data))
    }
}

fn load_input_data() -> Vec<u8> {
    let group_data_len = QueryIter::new(load_cell_data, Source::GroupInput).count();
    if group_data_len != 1 {
        panic!("inputs have more than 1 bridge cell");
    }
    load_cell_data(0, Source::GroupInput).unwrap()
}
