#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
extern crate no_std_compat as std;

pub mod actions;
pub mod adapter;

use adapter::Adapter;

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    let chain = contracts_helper::chain::Chain {};
    let adapter = adapter::ChainAdapter { chain };
    _verify(adapter);
    0
}

pub fn _verify<T: Adapter>(data_loader: T) {
    actions::verify_manage_mode(&data_loader);
}
