#[cfg(target_arch = "riscv64")]
pub mod chain;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32, Script},
    prelude::Pack,
};
use ckb_std::error::SysError;
use force_eth_types::config::SUDT_CODE_HASH;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;
use molecule::prelude::{Builder, Entity};
use std::prelude::v1::*;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_input_data(&self) -> Vec<u8>;

    fn load_script_hash(&self) -> [u8; 32];

    fn load_input_witness_args(&self) -> Result<Bytes, SysError>;

    fn load_cell_dep_data(&self, index: usize) -> Result<Vec<u8>, SysError>;

    /// check whether there is any input lock hash matches the given one
    fn lock_hash_exists_in_inputs(&self, hash: &[u8]) -> bool;

    /// check whether there is any output lock script matches the given one
    fn typescript_exists_in_outputs(&self, script: &[u8]) -> bool;

    fn outpoint_exists_in_inputs(&self, outpoint: &[u8]) -> bool;

    /// load cell type, lock, data at the same time.
    fn load_cell_type_lock_data(
        &self,
        index: usize,
        source: Source,
    ) -> Result<(Option<Script>, Script, Vec<u8>), SysError>;

    fn get_associated_udt_script(&self) -> Script {
        let script_hash = self.load_script_hash();
        Script::new_builder()
            .code_hash(Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
            .args(Bytes::from(script_hash.to_vec()).pack())
            .build()
    }
}
