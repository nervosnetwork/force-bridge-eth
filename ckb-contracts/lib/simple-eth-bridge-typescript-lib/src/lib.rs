#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
extern crate no_std_compat as std;

pub mod actions;
pub mod adapter;
#[cfg(test)]
mod test;

use adapter::Adapter;

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    let chain = contracts_helper::chain::Chain {};
    let adapter = adapter::ChainAdapter { chain };
    _verify(adapter);
    0
}

pub fn _verify<T: Adapter>(data_loader: T) {
    let input_data_size = data_loader.load_input_data_size();
    // if the script does not exist in input, ignore the check.
    // which enables user to create bridge cell with this typescript.
    if input_data_size == 0 {
        return;
    }
    actions::verify_manage_mode(&data_loader);
}
