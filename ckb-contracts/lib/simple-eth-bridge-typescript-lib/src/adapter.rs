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
    fn load_script_args(&self) -> Bytes;

    /// check whether there is any input lock script matches the given one
    fn lock_script_exists_in_inputs(&self, hash: &[u8]) -> bool;
}

pub struct ChainAdapter<T: DataLoader> {
    pub chain: T,
}

impl<T> Adapter for ChainAdapter<T>
where
    T: DataLoader,
{
    fn load_script_args(&self) -> Bytes {
        let args = self.chain.load_script().unwrap().args().raw_data();
    }

    fn lock_script_exists_in_inputs(&self, data: &[u8]) -> bool {
        QueryIter::new(
            |index, source| self.chain.load_cell_lock(index, source),
            Source::Input,
        )
        .any(|script| script.as_slice() == data)
    }
}
