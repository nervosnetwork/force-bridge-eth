use crate::adapter::Adapter;
use crate::debug;
use ckb_std::ckb_constants::Source;

use ckb_types::{
    core::DepType,
    packed::{Byte32, Bytes, Script},
};
use force_eth_types::{
    config::ETH_BRIDGE_LOCKSCRIPT_CODE_HASH, eth_recipient_cell::ETHRecipientDataView,
};
use molecule::prelude::{Builder, Entity};

pub fn verify_burn_token<T: Adapter>(data_loader: T, data: ETHRecipientDataView) {
    let eth_bridge_lock_hash = calc_eth_bridge_lock_hash(data.eth_token_address);
    let input_sudt_num =
        data_loader.get_sudt_amount_from_source(Source::Input, eth_bridge_lock_hash.as_slice());
    let output_sudt_num =
        data_loader.get_sudt_amount_from_source(Source::Output, eth_bridge_lock_hash.as_slice());
    if input_sudt_num < output_sudt_num {
        panic!("input sudt less than output sudt")
    }
    if input_sudt_num - output_sudt_num != data.token_amount {
        panic!("burned token amount invalid")
    }
}

fn calc_eth_bridge_lock_hash(eth_token_address: molecule::bytes::Bytes) -> Byte32 {
    debug!("eth_token_address {:?}", eth_token_address);
    let eth_bridge_lockscript = Script::new_builder()
        .code_hash(
            Byte32::from_slice(&ETH_BRIDGE_LOCKSCRIPT_CODE_HASH)
                .expect("eth bridge lockscript hash invalid"),
        )
        .hash_type(DepType::Code.into())
        .args(Bytes::new_unchecked(eth_token_address))
        .build();
    eth_bridge_lockscript.calc_script_hash()
}
