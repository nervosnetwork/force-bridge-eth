#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use super::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32, Script},
    prelude::Pack,
};
use ckb_std::error::SysError;
use ckb_std::high_level::{
    load_cell_data, load_cell_lock, load_cell_lock_hash, load_cell_type, load_cell_type_hash,
    load_script, load_script_hash, load_witness_args, QueryIter,
};
use force_eth_types::config::SUDT_CODE_HASH;
use force_eth_types::generated::eth_bridge_type_cell::{
    ETHBridgeTypeArgs, ETHBridgeTypeArgsReader, ETHBridgeTypeData,
};
use molecule::prelude::{Builder, Entity, Reader};

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
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

    fn load_cell_lock_script(&self, index: usize, source: Source) -> Result<Script, SysError> {
        load_cell_lock(index, source)
    }

    fn load_cell_data(&self, index: usize, source: Source) -> Result<Vec<u8>, SysError> {
        load_cell_data(index, source)
    }

    fn lock_script_exists_in_inputs(&self, data: &[u8]) -> bool {
        QueryIter::new(load_cell_lock, Source::Input).any(|script| script.as_slice() == data)
    }

    fn get_associated_udt_script(&self, bridge_lock_hash: &[u8]) -> Script {
        Script::new_builder()
            .code_hash(Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
            .args(Bytes::from(bridge_lock_hash).pack())
            .build()
    }

    fn load_data(&self) -> Option<ETHBridgeTypeData> {
        let data_list = QueryIter::new(load_cell_data, Source::GroupInput).collect::<Vec<_>>();
        match data_list.len() {
            0 => return None,
            1 => Some(ETHBridgeTypeData::from_slice(&data_list[0]).expect("invalid data")),
            _ => panic!("can not have more than one cell with this typescript"),
        }
    }
}
