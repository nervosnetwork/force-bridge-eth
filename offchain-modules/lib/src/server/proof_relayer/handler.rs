use crate::server::proof_relayer::db::{update_eth_to_ckb_status, EthToCkbRecord};
use crate::transfer::to_ckb::generate_eth_spv_proof_json;
use crate::transfer::to_eth::{get_ckb_proof_info, unlock, wait_block_submit};
use crate::util::ckb_util::Generator;
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_eth_address, convert_hex_to_h256, Web3Client};
use anyhow::{anyhow, Result};
use ckb_sdk::AddressPayload;
use ckb_sdk::SECP256K1;
use ckb_types::core::TransactionView;
use ckb_types::packed::Script;
use force_sdk::tx_helper::sign;
use molecule::prelude::Entity;
use secp256k1::SecretKey;
use sqlx::SqlitePool;

pub async fn relay_ckb_to_eth_proof(
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
    let (tx_proof, tx_info) = get_ckb_proof_info(&ckb_tx_hash, ckb_rpc_url.clone())?;
    let light_client = convert_eth_address(&deployed_contracts.eth_ckb_chain_addr)?;
    let lock_contract_addr = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    wait_block_submit(
        ethereum_rpc_url.clone(),
        ckb_rpc_url,
        light_client,
        ckb_tx_hash,
        lock_contract_addr,
    )
    .await?;
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
    log::info!("unlock result: {:?}", result);
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
    for i in 0u8..100 {
        let receipt_res = web3.get_receipt(eth_lock_tx_hash).await;
        match receipt_res {
            Ok(Some(receipt)) => {
                log::info!("get tx {} receipt: {:?}", eth_lock_tx_hash, receipt);
                break;
            }
            _ => {
                log::error!(
                    "tx {} not committed on eth yet, retry_index: {}",
                    eth_lock_tx_hash,
                    i
                );
                tokio::time::delay_for(std::time::Duration::from_secs(15)).await;
            }
        }
    }
    // generate proof and send tx
    let eth_proof = generate_eth_spv_proof_json(
        record.eth_lock_tx_hash.clone(),
        ethereum_rpc_url.clone(),
        eth_token_locker_addr.clone(),
    )
    .await?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);
    let unsigned_tx =
        generator.generate_eth_spv_tx(config_path.clone(), from_lockscript, &eth_proof)?;
    let tx =
        sign(unsigned_tx, &mut generator.rpc_client, &from_privkey).map_err(|err| anyhow!(err))?;
    let tx_hash = generator
        .rpc_client
        .send_transaction(tx.data())
        .map_err(|err| anyhow!("Send transaction error: {}", err))?;
    record.token_addr = Some(hex::encode(eth_proof.token.as_bytes()));
    record.ckb_recipient_lockscript = Some(hex::encode(eth_proof.recipient_lockscript));
    update_eth_to_ckb_status(db, &record).await?;
    for i in 0u8..100 {
        let status = generator
            .rpc_client
            .get_transaction(tx_hash.clone())
            .map_err(|e| anyhow!("get tx err: {}", e))?
            .map(|t| t.tx_status.status);
        log::info!(
            "waiting for tx {} to be committed, loop index: {}, status: {:?}",
            &tx_hash,
            i,
            status
        );
        if status == Some(ckb_jsonrpc_types::Status::Committed) {
            break;
        }
        tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
    }
    log::info!("relay tx {} successfully", tx_hash);
    // save result to db
    record.status = "success".to_owned();
    record.ckb_tx_hash = Some(tx_hash.to_string());
    update_eth_to_ckb_status(db, &record).await?;
    Ok(())
}
