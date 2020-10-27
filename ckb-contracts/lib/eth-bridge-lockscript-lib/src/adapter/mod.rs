#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::error::SysError;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Default, Clone)]
pub struct ComplexData {}

pub struct BridgeCellDataTuple(pub Option<Vec<u8>>, pub Option<Vec<u8>>);

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError>;

    fn load_input_output_cell_num(&self) -> (usize, usize);

    fn load_input_output_data(&self) -> Result<BridgeCellDataTuple, SysError>;

    fn get_complex_data(&self) -> Result<ComplexData, SysError>;
}
