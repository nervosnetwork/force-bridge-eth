use super::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32, CellOutput, Script},
    prelude::Pack,
};
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell, load_cell_data, load_cell_lock_hash, load_cell_type, load_cell_type_hash, load_input_out_point, load_script, load_script_hash, load_witness_args, QueryIter, load_cell_capacity};
use molecule::prelude::{Builder, Entity, Reader};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use force_eth_types::generated::eth_bridge_type_cell::{
    ETHBridgeTypeArgs, ETHBridgeTypeArgsReader,
};
use force_eth_types::config::SUDT_CODE_HASH;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn get_group_input_num(&self) -> usize {
        QueryIter::new(load_cell_capacity, Source::GroupInput).count()
    }

    fn load_input_data(&self) -> Vec<u8> {
        load_input_data()
    }

    fn load_script_hash(&self) -> [u8; 32] {
        load_script_hash().unwrap()
    }

    fn load_input_witness_args(&self) -> Result<Bytes, SysError> {
        let witness_args = load_witness_args(0, Source::GroupInput)?
            .lock()
            .to_opt()
            .expect("proof witness is none");
        Ok(witness_args.raw_data())
    }

    fn load_script_args(&self) -> ETHBridgeTypeArgs {
        let args = load_script().unwrap().args().raw_data();
        ETHBridgeTypeArgsReader::verify(args.as_ref(), false).expect("invalid script args");
        ETHBridgeTypeArgs::new_unchecked(args)
    }

    fn load_cell_dep_data(&self, index: usize) -> Result<Vec<u8>, SysError> {
        self.load_cell_data(index, Source::CellDep)
    }

    fn load_cell(&self, index: usize, source: Source) -> Result<CellOutput, SysError> {
        load_cell(index, source)
    }

    fn load_cell_type(&self, index: usize, source: Source) -> Result<Option<Script>, SysError> {
        load_cell_type(index, source)
    }

    fn load_cell_type_hash(
        &self,
        index: usize,
        source: Source,
    ) -> Result<Option<[u8; 32]>, SysError> {
        load_cell_type_hash(index, source)
    }

    fn load_cell_lock_hash(&self, index: usize, source: Source) -> Result<[u8; 32], SysError> {
        load_cell_lock_hash(index, source)
    }

    fn load_cell_data(&self, index: usize, source: Source) -> Result<Vec<u8>, SysError> {
        load_cell_data(index, source)
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

    fn get_associated_udt_script(&self, bridge_lock_hash: &[u8]) -> Script {
        Script::new_builder()
            .code_hash(Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
            .args(Bytes::from(bridge_lock_hash).pack())
            .build()
    }
}

fn load_input_data() -> Vec<u8> {
    let data_list = QueryIter::new(load_cell_data, Source::GroupInput).collect::<Vec<Vec<u8>>>();
    if data_list.len() != 1 {
        panic!("inputs have more than 1 bridge cell");
    }
    data_list[0].clone()
}
