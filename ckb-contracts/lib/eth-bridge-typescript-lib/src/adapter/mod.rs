#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{bytes::Bytes, packed::Script};
use ckb_std::error::SysError;
use force_eth_types::generated::eth_bridge_type_cell::{ETHBridgeTypeArgs, ETHBridgeTypeData};

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
