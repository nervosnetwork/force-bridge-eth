#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32, CellOutput, Script},
    prelude::Pack,
};
use ckb_std::error::SysError;
use force_eth_types::config::SUDT_CODE_HASH;
use force_eth_types::generated::eth_bridge_type_cell::ETHBridgeTypeArgs;
use molecule::prelude::{Builder, Entity};

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    // fn load_input_output_data(&self) -> Result<BridgeCellDataTuple, SysError>;

    fn load_input_data(&self) -> Vec<u8>;

    fn load_script_hash(&self) -> [u8; 32];

    fn load_input_witness_args(&self) -> Result<Bytes, SysError>;

    fn load_script_args(&self) -> ETHBridgeTypeArgs;

    fn load_cell_dep_data(&self, index: usize) -> Result<Vec<u8>, SysError>;

    fn load_cell(&self, index: usize, source: Source) -> Result<CellOutput, SysError>;

    fn load_cell_type(&self, index: usize, source: Source) -> Result<Option<Script>, SysError>;

    fn load_cell_type_hash(
        &self,
        index: usize,
        source: Source,
    ) -> Result<Option<[u8; 32]>, SysError>;

    fn load_cell_lock_hash(&self, index: usize, source: Source) -> Result<[u8; 32], SysError>;

    fn load_cell_data(&self, index: usize, source: Source) -> Result<Vec<u8>, SysError>;

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
