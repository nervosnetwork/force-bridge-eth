#![cfg_attr(not(feature = "std"), no_std)]

extern crate no_std_compat as std;

pub mod actions;
pub mod adapter;
pub mod debug;
#[cfg(test)]
mod test;

use adapter::Adapter;
use adapter::BridgeCellDataTuple;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    let chain = adapter::chain::ChainAdapter {};
    _verify(chain);
    0
}

pub fn _verify<T: Adapter>(data_loader: T) {
    let mode = actions::check_mode(&data_loader);
    match mode {
        actions::Mode::Owner => {
            actions::verify_owner_mode(&data_loader);
        }
        actions::Mode::Mint => {
            actions::verify_mint_token(&data_loader);
        }
    }
}
