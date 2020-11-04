#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::error::SysError;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct BridgeCellDataTuple(pub Option<Vec<u8>>, pub Option<Vec<u8>>);

use molecule::bytes::Bytes;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_input_output_data(&self) -> Result<BridgeCellDataTuple, SysError>;

    fn load_input_witness_args(&self) -> Result<Bytes, SysError>;

    fn load_cell_dep_data(&self, index: usize) -> Result<Vec<u8>, SysError>;

    fn check_inputs_lock_hash(&self, data: &[u8]) -> bool;
}
