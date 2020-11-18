#![cfg_attr(not(feature = "std"), no_std)]

extern crate no_std_compat as std;

pub mod actions;
pub mod adapter;
pub mod debug;

use adapter::Adapter;
use force_eth_types::generated::witness::MintTokenWitnessReader;
use molecule::prelude::{Entity, Reader};

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
    // if the script does not exist in input, ignore the check.
    // which enables user to create bridge cell with this typescript.
    if data_loader.get_group_input_num() == 0 {
        return;
    }
    // load and parse witness
    let witness_args = data_loader
        .load_input_witness_args()
        .expect("load witness args error");
    MintTokenWitnessReader::verify(&witness_args, false).expect("witness is invalid");
    let witness = MintTokenWitnessReader::new_unchecked(&witness_args);
    debug!("witness: {:?}", witness);

    // load script args
    let script_args = data_loader.load_script_args();

    // check mode
    let mode: u8 = witness.mode().into();
    match mode {
        0 => {
            actions::verify_mint_token(&data_loader, &witness, &script_args);
        }
        _ => {
            actions::verify_manage_mode(&data_loader, script_args.owner_lock_hash().as_slice());
        }
    }
}
