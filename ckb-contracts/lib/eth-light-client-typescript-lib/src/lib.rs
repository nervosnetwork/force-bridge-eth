#![cfg_attr(not(feature = "std"), no_std)]

mod actions;
mod adapter;

#[cfg(not(feature = "std"))]
extern crate alloc;
extern crate no_std_compat as std;

pub use adapter::Adapter;

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    let chain = contracts_helper::chain::Chain {};
    let adapter = adapter::ChainAdapter { chain };
    _verify(adapter);
    0
}

pub fn _verify<T: Adapter>(adapter: T) {
    actions::verify(adapter)
}
