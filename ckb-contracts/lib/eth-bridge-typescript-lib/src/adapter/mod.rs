#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32, Script},
    prelude::Pack,
};
use ckb_std::error::SysError;
use ckb_std::high_level::QueryIter;
use contracts_helper::data_loader::DataLoader;
use force_eth_types::config::{SUDT_CODE_HASH, SUDT_HASH_TYPE};
use force_eth_types::generated::eth_bridge_type_cell::{
    ETHBridgeTypeArgs, ETHBridgeTypeArgsReader, ETHBridgeTypeData,
};
use molecule::prelude::{Builder, Entity, Reader};
use std::prelude::v1::*;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_script_hash(&self) -> [u8; 32];

    fn load_input_witness_args(&self) -> Result<Bytes, SysError>;

    fn load_script_args(&self) -> ETHBridgeTypeArgs;

    fn load_data(&self) -> Option<ETHBridgeTypeData>;

    fn load_cell_type(&self, index: usize, source: Source) -> Result<Option<Script>, SysError>;

    fn load_cell_type_hash(
        &self,
        index: usize,
        source: Source,
    ) -> Result<Option<[u8; 32]>, SysError>;

    fn load_cell_lock_hash(&self, index: usize, source: Source) -> Result<[u8; 32], SysError>;

    fn load_cell_lock_script(&self, index: usize, source: Source) -> Result<Script, SysError>;

    fn load_cell_data(&self, index: usize, source: Source) -> Result<Vec<u8>, SysError>;

    /// check whether there is any input lock script matches the given one
    fn lock_script_exists_in_inputs(&self, hash: &[u8]) -> bool;

    fn get_associated_udt_script(&self, bridge_lock_hash: &[u8]) -> Script;
}

pub struct ChainAdapter<T: DataLoader> {
    pub chain: T,
}

impl<T> Adapter for ChainAdapter<T>
where
    T: DataLoader,
{
    fn load_script_hash(&self) -> [u8; 32] {
        self.chain.load_script_hash().unwrap()
    }

    fn load_input_witness_args(&self) -> Result<Bytes, SysError> {
        let witness_args = self
            .chain
            .load_witness_args(0, Source::GroupInput)?
            .lock()
            .to_opt()
            .expect("proof witness is none");
        Ok(witness_args.raw_data())
    }

    fn load_script_args(&self) -> ETHBridgeTypeArgs {
        let args = self.chain.load_script().unwrap().args().raw_data();
        ETHBridgeTypeArgsReader::verify(args.as_ref(), false).expect("invalid script args");
        ETHBridgeTypeArgs::new_unchecked(args)
    }

    fn load_data(&self) -> Option<ETHBridgeTypeData> {
        let data_list = QueryIter::new(
            |index, source| self.chain.load_cell_data(index, source),
            Source::GroupInput,
        )
        .collect::<Vec<_>>();
        match data_list.len() {
            0 => return None,
            1 => Some(ETHBridgeTypeData::from_slice(&data_list[0]).expect("invalid data")),
            _ => panic!("can not have more than one cell with this typescript"),
        }
    }

    fn load_cell_type(&self, index: usize, source: Source) -> Result<Option<Script>, SysError> {
        self.chain.load_cell_type(index, source)
    }

    fn load_cell_type_hash(
        &self,
        index: usize,
        source: Source,
    ) -> Result<Option<[u8; 32]>, SysError> {
        self.chain.load_cell_type_hash(index, source)
    }

    fn load_cell_lock_hash(&self, index: usize, source: Source) -> Result<[u8; 32], SysError> {
        self.chain.load_cell_lock_hash(index, source)
    }

    fn load_cell_lock_script(&self, index: usize, source: Source) -> Result<Script, SysError> {
        self.chain.load_cell_lock_script(index, source)
    }

    fn load_cell_data(&self, index: usize, source: Source) -> Result<Vec<u8>, SysError> {
        self.chain.load_cell_data(index, source)
    }

    fn lock_script_exists_in_inputs(&self, data: &[u8]) -> bool {
        QueryIter::new(
            |index, source| self.chain.load_cell_lock(index, source),
            Source::Input,
        )
        .any(|script| script.as_slice() == data)
    }

    fn get_associated_udt_script(&self, bridge_lock_hash: &[u8]) -> Script {
        Script::new_builder()
            .code_hash(Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
            .hash_type(SUDT_HASH_TYPE.into())
            .args(Bytes::from(bridge_lock_hash.to_vec()).pack())
            .build()
    }
}
