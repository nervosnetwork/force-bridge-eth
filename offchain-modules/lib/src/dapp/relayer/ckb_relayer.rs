use crate::transfer::to_eth::{get_ckb_proof_info, unlock};
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_eth_address, parse_private_key, Web3Client};
use anyhow::{anyhow, Result};
use ethereum_types::{H160, H256};
use futures::future::join_all;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::ops::Add;
use std::time::Instant;
use tokio::time::Duration;

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UnlockTask {
    pub id: u32,
    pub ckb_burn_tx_hash: String,
    // pub ckb_spv_proof: String,
    pub ckb_raw_tx: String,
}

pub struct CkbTxRelay {
    eth_token_locker_addr: String,
    ethereum_rpc_url: String,
    ckb_rpc_url: String,
    rocksdb_path: String,
    eth_private_key: H256,
    web3_client: Web3Client,
    contract_addr: H160,
    confirm_num: u64,
    db: MySqlPool,
}

impl CkbTxRelay {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        db_path: String,
        private_key_path: String,
        rocksdb_path: String,
    ) -> Result<CkbTxRelay> {
        let force_config = ForceConfig::new(&config_path)?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;
        let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let db = MySqlPool::connect(&db_path).await?;
        let eth_private_key = parse_private_key(&private_key_path, &force_config, &network)?;
        let eth_token_locker_addr = deployed_contracts.eth_token_locker_addr.clone();
        let contract_addr = convert_eth_address(&deployed_contracts.eth_ckb_chain_addr.clone())?;
        let token_locker_addr = convert_eth_address(&eth_token_locker_addr)?;
        let mut web3_client = Web3Client::new(ethereum_rpc_url.clone());
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let confirm_num = web3_client
            .get_locker_contract_confirm("numConfirmations_", token_locker_addr)
            .await?;
        Ok(CkbTxRelay {
            eth_token_locker_addr,
            ethereum_rpc_url,
            ckb_rpc_url,
            rocksdb_path,
            eth_private_key,
            web3_client,
            contract_addr,
            confirm_num,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            self.relay().await?;
            tokio::time::delay_for(Duration::from_secs(60)).await
        }
    }

    pub async fn relay(&mut self) -> Result<()> {
        let client_block_number = self
            .web3_client
            .get_contract_height("latestBlockNumber", self.contract_addr)
            .await?;
        let unlock_tasks =
            get_unlock_tasks(&self.db, self.confirm_num, client_block_number).await?;
        let mut unlock_futures = vec![];
        let nonce = self
            .web3_client
            .get_eth_nonce(&self.eth_private_key)
            .await?;
        for (i, tx_record) in unlock_tasks.iter().enumerate() {
            info!("burn tx wait to unlock: {:?} ", tx_record.ckb_burn_tx_hash);
            let proof_info = get_ckb_proof_info(
                &tx_record.ckb_burn_tx_hash,
                self.ckb_rpc_url.clone(),
                String::from(self.web3_client.url()),
                self.contract_addr,
                self.rocksdb_path.clone(),
            )
            .await?;
            unlock_futures.push(unlock(
                self.eth_private_key,
                self.ethereum_rpc_url.clone(),
                self.eth_token_locker_addr.clone(),
                proof_info,
                0,
                nonce.add(i),
                true,
            ));
        }
        if !unlock_futures.is_empty() {
            let now = Instant::now();
            let unlock_count = unlock_futures.len();

            let timeout_future = tokio::time::delay_for(std::time::Duration::from_secs(1800));
            let task_future = join_all(unlock_futures);
            tokio::select! {
                v = task_future => {
                    for res in v.iter() {
                       match res {
                          Ok(hash) => info!("unlock hash : {}", hash),
                          Err(error) => error!("unlock error : {:?}", error),
                    }
                  }
                  info!("unlock {} txs elapsed {:?}", unlock_count, now.elapsed());
               }
                _ = timeout_future => {
                    error!("batch relay ckb tx timeout");
                }
            }
        }
        Ok(())
    }
}

pub async fn get_unlock_tasks(
    pool: &MySqlPool,
    confirm: u64,
    height: u64,
) -> Result<Vec<UnlockTask>> {
    let sql = r#"
SELECT id, ckb_burn_tx_hash, ckb_raw_tx
FROM ckb_to_eth
WHERE status = 'pending' AND ckb_block_number + ? < ?
    "#;
    let tasks = sqlx::query_as::<_, UnlockTask>(sql)
        .bind(confirm)
        .bind(height)
        .fetch_all(pool)
        .await?;
    Ok(tasks)
}
