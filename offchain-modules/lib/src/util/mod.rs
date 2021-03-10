pub mod ckb_proof_helper;
pub mod ckb_tx_generator;
pub mod ckb_types;
pub mod ckb_util;
pub mod config;
pub mod eth_proof_helper;
pub mod eth_util;
pub mod generated;
pub mod rocksdb;

use self::config::ForceConfig;
use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{parse_privkey, parse_privkey_path};
use ::ckb_types::core::Capacity;
use ::ckb_types::packed::{CellOutput, Script};
use ::ckb_types::prelude::{Builder, Entity, Pack};
use anyhow::{anyhow, Result};
use ckb_sdk::{Address, HumanCapacity};
use force_sdk::tx_helper::TxHelper;
use force_sdk::util::ensure_indexer_sync;
use std::str::FromStr;

#[allow(clippy::too_many_arguments)]
pub async fn transfer(
    config_path: String,
    network: Option<String>,
    privkey_path: String,
    to_addr: String,
    ckb_amount: String,
    tx_fee: String,
) -> Result<String> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;

    let mut generator =
        Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default()).map_err(|e| anyhow!(e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .await
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
    let from_privkey = parse_privkey_path(&privkey_path, &force_config, &network)?;
    let from_lockscript = parse_privkey(&from_privkey);
    let to_lockscript: Script = Address::from_str(&to_addr)
        .map_err(|e| anyhow!("failed to covert address  : {}", e))?
        .payload()
        .into();
    let tx_fee: u64 = HumanCapacity::from_str(&tx_fee)
        .map_err(|e| anyhow!(e))?
        .into();
    let ckb_amount: u64 = HumanCapacity::from_str(&ckb_amount)
        .map_err(|e| anyhow!(e))?
        .into();

    let mut helper = TxHelper::default();
    let recipient_output = CellOutput::new_builder()
        .capacity(Capacity::shannons(ckb_amount).pack())
        .lock(to_lockscript)
        .build();
    helper.add_output(recipient_output, Default::default());
    // add signature to pay tx fee
    let unsigned_tx = helper
        .supply_capacity(
            &mut generator.rpc_client,
            &mut generator.indexer_client,
            from_lockscript,
            &generator.genesis_info,
            tx_fee,
        )
        .map_err(|err| anyhow!(err))?;
    generator
        .sign_and_send_transaction(unsigned_tx, from_privkey)
        .await
}
