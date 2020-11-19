#[cfg(target_arch = "riscv64")]
pub mod chain;
#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::ckb_constants::Source;

use force_eth_types::eth_recipient_cell::ETHRecipientDataView;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_output_data(&self) -> Option<ETHRecipientDataView>;

    fn get_sudt_amount_from_source(&self, source: Source, lock_hash: &[u8]) -> u128;
}
