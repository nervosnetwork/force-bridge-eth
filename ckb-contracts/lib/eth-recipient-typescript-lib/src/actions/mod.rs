use crate::adapter::Adapter;
use crate::debug;
use ckb_std::ckb_constants::Source;

use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::ckb_types::packed::{Byte32, Bytes, Script};
use core::convert::TryFrom;
use force_eth_types::{
    config::ETH_BRIDGE_LOCKSCRIPT_CODE_HASH,
    eth_recipient_cell::{ETHAddress, ETHRecipientDataView},
    generated::eth_bridge_lock_cell::ETHBridgeLockArgs,
};
use molecule::prelude::{Builder, Byte, Entity};

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub fn verify_burn_token<T: Adapter>(data_loader: T, data: ETHRecipientDataView) {
    let eth_contract_address = ETHAddress::try_from(data_loader.load_script_args().to_vec())
        .expect("eth_contract_address in witness length wrong");
    let eth_bridge_lock_hash =
        calc_eth_bridge_lock_hash(eth_contract_address, data.eth_token_address);
    let input_sudt_num =
        data_loader.get_sudt_amount_from_source(Source::Input, &eth_bridge_lock_hash);
    let output_sudt_num =
        data_loader.get_sudt_amount_from_source(Source::Output, &eth_bridge_lock_hash);
    if input_sudt_num < output_sudt_num {
        panic!("input sudt less than output sudt")
    }
    if input_sudt_num - output_sudt_num != data.token_amount {
        panic!("burned token amount invalid")
    }

    if data.fee >= data.token_amount {
        panic!("fee is too much")
    }
}

fn calc_eth_bridge_lock_hash(
    eth_contract_address: ETHAddress,
    eth_token_address: ETHAddress,
) -> [u8; 32] {
    debug!("eth_token_address {:?}", eth_token_address);
    let args = ETHBridgeLockArgs::new_builder()
        .eth_contract_address(eth_contract_address.get_address().into())
        .eth_token_address(eth_token_address.get_address().into())
        .build();
    let eth_bridge_lockscript = Script::new_builder()
        .code_hash(
            Byte32::from_slice(&ETH_BRIDGE_LOCKSCRIPT_CODE_HASH)
                .expect("eth bridge lockscript hash invalid"),
        )
        .hash_type(Byte::new(0))
        .args(Bytes::new_unchecked(args.as_bytes()))
        .build();

    blake2b_256(eth_bridge_lockscript.as_slice())
}

fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

fn blake2b_256(s: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(s);
    blake2b.finalize(&mut result);
    result
}
