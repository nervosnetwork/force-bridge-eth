#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
extern crate no_std_compat as std;

use contracts_helper::data_loader::DataLoader;
use contracts_helper::debug;
use force_eth_types::generated::witness::MintTokenWitnessReader;
use molecule::prelude::{Entity, Reader};

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    _verify(adapter);
    0
}

pub fn _verify<T: DataLoader>(data_loader: T) {
    // todo
}
