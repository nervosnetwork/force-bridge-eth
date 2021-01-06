use crate::dapp::server::proof_relayer::db::{
    self, update_eth_to_ckb_status, CkbToEthRecord, EthToCkbRecord,
};
use crate::header_relay::eth_relay::wait_header_sync_success;
use crate::transfer::to_ckb::{generate_eth_spv_proof_json, send_eth_spv_proof_tx};
use crate::transfer::to_eth::{get_ckb_proof_info, unlock, wait_block_submit};
use crate::util::ckb_tx_generator::Generator;
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_eth_address, convert_hex_to_h256, Web3Client};
use anyhow::{anyhow, bail, Result};
use ckb_jsonrpc_types::Uint128;
use ckb_types::core::TransactionView;
use molecule::prelude::Entity;
use secp256k1::SecretKey;
use sqlx::SqlitePool;

pub async fn relay_ckb_to_eth_proof(
    mut record: CkbToEthRecord,
    db: &SqlitePool,
    config_path: String,
    eth_privkey_path: String,
    network: Option<String>,
    tx: TransactionView,
) -> Result<()> {
    let ckb_tx_hash = hex::encode(tx.hash().as_slice());
    let force_config = ForceConfig::new(config_path.as_str())?;
    let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let try_get_ckb_proof = async || {
        let mut error = "".to_string();
        for _ in 0..3 {
            let ret = get_ckb_proof_info(&ckb_tx_hash, ckb_rpc_url.clone());
            if ret.is_ok() {
                return ret;
            }
            error = format!("{}", ret.unwrap_err());
            tokio::time::delay_for(std::time::Duration::from_secs(60)).await;
        }
        bail!("get ckb burn tx proof failed: {}", error);
    };
    let (tx_proof, tx_info) = try_get_ckb_proof().await?;

    let light_client = convert_eth_address(&deployed_contracts.eth_ckb_chain_addr)?;
    let lock_contract_addr = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;

    let timeout_future = tokio::time::delay_for(std::time::Duration::from_secs(3600));
    let wait_header_future = wait_block_submit(
        ethereum_rpc_url.clone(),
        ckb_rpc_url,
        light_client,
        ckb_tx_hash.clone(),
        lock_contract_addr,
    );
    tokio::select! {
        v = wait_header_future => { v? }
        _ = timeout_future => {
            bail!("wait header sync timeout");
        }
    }

    let result = unlock(
        config_path,
        network,
        eth_privkey_path,
        deployed_contracts.eth_token_locker_addr.clone(),
        tx_proof,
        tx_info,
        0,
        true,
    )
    .await?;
    record.eth_tx_hash = Some(format!("0x{}", &result));
    record.status = "success".into();
    db::update_ckb_to_eth_status(db, &record).await?;
    log::info!("burn tx: {:?}, unlock succeed: {:?}", &ckb_tx_hash, &result);
    Ok(())
}

pub async fn relay_eth_to_ckb_proof(
    mut record: EthToCkbRecord,
    ethereum_rpc_url: String,
    eth_token_locker_addr: String,
    mut generator: Generator,
    config_path: String,
    from_privkey: SecretKey,
    db: &SqlitePool,
) -> Result<()> {
    let mut web3 = Web3Client::new(ethereum_rpc_url.clone());
    let eth_lock_tx_hash = convert_hex_to_h256(&record.eth_lock_tx_hash)?;

    // ensure tx committed on eth
    let mut is_committed = false;
    for i in 0u8..100 {
        let receipt_res = web3.get_receipt(eth_lock_tx_hash).await;
        match receipt_res {
            Ok(Some(receipt)) => {
                log::info!("get lock tx {} receipt: {:?}", eth_lock_tx_hash, receipt);
                is_committed = true;
                break;
            }
            _ => {
                log::error!(
                    "lock tx {} not committed on eth yet, retry_index: {}",
                    eth_lock_tx_hash,
                    i
                );
                tokio::time::delay_for(std::time::Duration::from_secs(15)).await;
            }
        }
    }
    if !is_committed {
        bail!("wait lock tx committed on ethereum timeout");
    }

    // generate proof and send tx
    let eth_proof = generate_eth_spv_proof_json(
        record.eth_lock_tx_hash.clone(),
        ethereum_rpc_url.clone(),
        eth_token_locker_addr.clone(),
    )
    .await?;
    let force_config = ForceConfig::new(config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;

    let timeout_future = tokio::time::delay_for(std::time::Duration::from_secs(1800));
    let wait_header_future = wait_header_sync_success(
        &mut generator,
        deployed_contracts
            .light_client_cell_script
            .cell_script
            .as_str(),
        eth_proof.header_data.clone(),
    );
    tokio::select! {
        v = wait_header_future => { v? }
        _ = timeout_future => {
            bail!("wait header sync timeout");
        }
    }

    let tx_hash = send_eth_spv_proof_tx(
        &mut generator,
        config_path,
        record.eth_lock_tx_hash.clone(),
        &eth_proof,
        from_privkey,
        None,
    )
    .await?;
    log::info!(
        "relay lock tx {} successfully, mint tx {}",
        eth_lock_tx_hash,
        tx_hash
    );
    // save result to db
    record.token_addr = Some(hex::encode(eth_proof.token.as_bytes()));
    record.ckb_recipient_lockscript = Some(hex::encode(eth_proof.recipient_lockscript));
    record.locked_amount = Some(Uint128::from(eth_proof.lock_amount).to_string());
    record.status = "success".to_owned();
    record.ckb_tx_hash = Some(format!("0x{}", tx_hash));
    update_eth_to_ckb_status(db, &record).await?;
    Ok(())
}
