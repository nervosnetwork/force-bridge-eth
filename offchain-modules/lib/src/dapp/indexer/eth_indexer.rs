use crate::dapp::db::db::{
    create_eth_to_ckb_record, get_latest_eth_to_ckb_record, is_eth_to_ckb_record_exist,
    EthToCkbRecord,
};
use crate::transfer::to_ckb::to_eth_spv_proof_json;
use crate::util::ckb_util::EthWitness;
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_hex_to_h256, Web3Client};
use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::Uint128;
use log::{debug, info};
use rusty_receipt_proof_maker::{generate_eth_proof, types::EthSpvProof};
use shellexpand::tilde;
use sqlx::MySqlPool;
use std::ops::Add;
use web3::types::U64;

pub struct EthIndexer {
    pub force_config: ForceConfig,
    pub eth_client: Web3Client,
    pub db: MySqlPool,
}

impl EthIndexer {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        db_path: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let db_path = tilde(db_path.as_str()).into_owned();
        let db = MySqlPool::connect(&db_path).await?;
        Ok(EthIndexer {
            force_config,
            eth_client,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let record_option = get_latest_eth_to_ckb_record(&self.db).await?;
        let mut start_block_number;
        if record_option.is_some() {
            let tx_hash_str = record_option.unwrap().eth_lock_tx_hash;
            let tx_hash = convert_hex_to_h256(&tx_hash_str)?;
            let receipt = self.eth_client.get_receipt(tx_hash).await?.unwrap();
            start_block_number = receipt.block_number.unwrap();
        } else {
            // start_block_number = self.eth_client.client().eth().block_number().await?;
            start_block_number = U64::from(332);
        }

        loop {
            let block = self.eth_client.get_block(start_block_number.into()).await;
            if block.is_err() {
                debug!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
                continue;
            }
            let txs = block.unwrap().transactions;
            for tx_hash in txs {
                let hash = hex::encode(tx_hash);
                let hash_with_0x = format!("{}{}", "0x", hash.clone());
                if is_eth_to_ckb_record_exist(&self.db, &hash).await? {
                    // the record is exist, check unlock event.
                    // let ret: Result<bool, AppError> =
                    //     parse_unlock_tx(hash, String::from(self.eth_client.url()));
                    // match ret {
                    //     Ok(ret) => {
                    //         if ret {
                    //             // update ckb to eth record status.
                    //             // unlock event add ckb_tx hash.
                    //         }
                    //     }
                    //     Err(_) => { // retry
                    //     }
                    // }
                    continue;
                }

                // let get_eth_spv_proof_with_retry = |max_retry_times| {};

                let (eth_spv_proof, success) =
                    self.get_eth_spv_proof_with_retry(hash_with_0x.clone(), 3)?;
                if success {
                    let lock_contract_address = self
                        .force_config
                        .deployed_contracts
                        .as_ref()
                        .unwrap()
                        .eth_token_locker_addr
                        .clone();
                    let eth_proof_json = to_eth_spv_proof_json(
                        hash_with_0x,
                        eth_spv_proof.clone(),
                        lock_contract_address,
                        String::from(self.eth_client.url()),
                    )
                    .await?;
                    let mut record = EthToCkbRecord {
                        eth_lock_tx_hash: hash.clone(),
                        status: "pending".to_string(),
                        token_addr: Some(hex::encode(eth_spv_proof.token.as_bytes())),
                        ckb_recipient_lockscript: Some(hex::encode(
                            eth_proof_json.recipient_lockscript.clone(),
                        )),
                        locked_amount: Some(Uint128::from(eth_spv_proof.lock_amount).to_string()),
                        ..Default::default()
                    };
                    let witness = EthWitness {
                        cell_dep_index_list: vec![0],
                        spv_proof: eth_proof_json,
                    }
                    .as_bytes();
                    record.eth_spv_proof = Some(witness.to_vec());
                    create_eth_to_ckb_record(&self.db, &record).await?;
                    info!("create eth_to_ckb record success. tx_hash: {}", hash,);
                }
            }
            start_block_number = start_block_number.add(U64::from(1));
        }
    }

    pub fn get_eth_spv_proof_with_retry(
        &mut self,
        hash: String,
        max_retry_times: i32,
    ) -> Result<(EthSpvProof, bool)> {
        for retry in 0..max_retry_times {
            let ret = generate_eth_proof(hash.clone(), String::from(self.eth_client.url()).clone());
            match ret {
                Ok(proof) => return Ok((proof, true)),
                Err(e) => {
                    info!(
                        "get eth receipt proof failed, retried {} times, err: {}",
                        retry, e
                    );
                    if e.to_string().contains("the locked tx is not exist") {
                        info!("the locked tx is not exist");
                        return Ok((Default::default(), false));
                    }
                }
            }
        }
        Err(anyhow!(
            "Failed to generate eth proof for lock tx:{}, after retry {} times",
            hash.as_str(),
            max_retry_times
        ))
    }
}
