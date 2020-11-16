#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::ckb_types::packed::Script;

use molecule::bytes::Bytes;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_input_script(&self) -> Script;

    fn load_output_script(&self) -> Script;

    fn check_input_owner(&self, owner_script: &Bytes) -> bool;
}
