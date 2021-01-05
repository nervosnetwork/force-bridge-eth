use crate::transfer::to_eth::unlock;
use crate::util::config::ForceConfig;
use crate::util::eth_util::{parse_private_key, Web3Client};
use anyhow::{anyhow, Result};
use ethereum_types::H256;
use futures::future::join_all;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::ops::Add;
use std::time::Instant;
use tokio::time::Duration;

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CkbToEthRecord {
    pub id: u32,
    pub ckb_burn_tx_hash: String,
    // pub status: String,
    // pub recipient_addr: Option<String>,
    // pub token_addr: Option<String>,
    // pub token_amount: Option<String>,
    // pub fee: Option<String>,
    // pub eth_tx_hash: Option<String>,
    // pub err_msg: Option<String>,
    pub ckb_spv_proof: Vec<u8>,
    pub ckb_raw_tx: Vec<u8>,
}

pub struct CkbTxRelay {
    eth_token_locker_addr: String,
    ethereum_rpc_url: String,
    eth_private_key: H256,
    db: MySqlPool,
}

impl CkbTxRelay {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        db_path: String,
        private_key_path: String,
    ) -> Result<CkbTxRelay> {
        let force_config = ForceConfig::new(&config_path)?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;
        let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let db = MySqlPool::connect(&db_path).await?;
        let eth_private_key = parse_private_key(&private_key_path, &force_config, &network)?;

        Ok(CkbTxRelay {
            eth_token_locker_addr: deployed_contracts.eth_token_locker_addr.clone(),
            ethereum_rpc_url,
            eth_private_key,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // let mut unlock_tasks: Vec<UnlockTask> = vec![];
        loop {
            self.relay().await?;
            tokio::time::delay_for(Duration::from_secs(600)).await
        }
    }

    pub async fn relay(&mut self) -> Result<()> {
        let unlock_tasks = get_ckb_tx_record(&self.db).await?;
        let mut unlock_futures = vec![];
        let nonce = Web3Client::new(self.ethereum_rpc_url.clone())
            .get_eth_nonce(&self.eth_private_key)
            .await?;
        for (i, tx_record) in unlock_tasks.iter().enumerate() {
            let tx_proof = hex::encode(tx_record.ckb_spv_proof.clone());
            let raw_tx = hex::encode(tx_record.ckb_raw_tx.clone());
            unlock_futures.push(unlock(
                self.eth_private_key,
                self.ethereum_rpc_url.clone(),
                self.eth_token_locker_addr.clone(),
                tx_proof,
                raw_tx,
                0,
                nonce.add(i),
                true,
            ));
        }
        if !unlock_futures.is_empty() {
            let now = Instant::now();
            let unlock_count = unlock_futures.len();

            let timeout_future = tokio::time::delay_for(std::time::Duration::from_secs(600));
            let task_future = join_all(unlock_futures);
            tokio::select! {
                v = task_future => {
                    for res in v.iter() {
                       match res {
                          Ok(hash) => info!("hash : {}", hash),
                          Err(error) => error!("error {:?}", error),
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

pub async fn get_ckb_tx_record(pool: &MySqlPool) -> Result<Vec<CkbToEthRecord>> {
    // TODO : filter with status
    Ok(sqlx::query_as::<_, CkbToEthRecord>(
        r#"
SELECT *
FROM ckb_to_eth
        "#,
    )
    .fetch_all(pool)
    .await?)
}
