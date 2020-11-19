#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use force_eth_types::eth_header_cell::ETHHeaderCellDataView;
use molecule::bytes::Bytes;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_data_from_source(&self, source: Source) -> Option<ETHHeaderCellDataView>;

    fn load_data_from_dep(&self, index: usize) -> Vec<u8>;

    fn load_witness_args(&self) -> Bytes;

    fn load_script_args(&self) -> Bytes;

    fn load_first_outpoint(&self) -> Bytes;
}
