use crate::transfer::to_eth::unlock;
use crate::util::config::ForceConfig;
use anyhow::{anyhow, Result};
use log::info;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use sqlx::MySqlPool;
use std::time::Instant;

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
    pub ckb_spv_proof: Option<Vec<u8>>,
    pub ckb_raw_tx: Option<Vec<u8>>,
}

pub struct CkbTxRelay {
    config_path: String,
    eth_token_locker_addr: String,
    network: Option<String>,
    ethereum_rpc_url: String,
    db: MySqlPool,
}

pub struct UnlockTask {
    pub burn_tx_proof: String,
    pub burn_raw_tx: String,
}

impl CkbTxRelay {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        db_path: String,
    ) -> Result<CkbTxRelay> {
        let force_config = ForceConfig::new(config_path.as_str())?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;
        let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let db_path = tilde(db_path.as_str()).into_owned();
        // let db_options = MySqlConnectOptions::from_str(&db_path).unwrap();
        let db = MySqlPool::connect(&db_path).await?;
        Ok(CkbTxRelay {
            config_path,
            eth_token_locker_addr: deployed_contracts.eth_token_locker_addr.clone(),
            network,
            ethereum_rpc_url,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let wait_relay_txs = get_ckb_tx_record(&self.db).await?;
        info!("data size : {}", wait_relay_txs.len());
        for tx in wait_relay_txs.iter() {
            let tx_proof;
            let raw_tx;
            match tx.ckb_spv_proof.clone() {
                Some(proof) => tx_proof = hex::encode(proof),
                None => continue,
            }
            match tx.ckb_raw_tx.clone() {
                Some(tx_info) => raw_tx = hex::encode(tx_info),
                None => continue,
            }
            info!(
                "tx proof : {:?} \n tx info {:?}",
                tx_proof.clone(),
                raw_tx.clone()
            );
            let result = unlock(
                self.config_path.clone(),
                self.network.clone(),
                "1".to_string(),
                self.eth_token_locker_addr.clone(),
                tx_proof,
                raw_tx,
                0,
                true,
            )
            .await?;
            info!("unlock hash : {}", result);
            //
            // let mut unlock_futures = vec![];
            // for task in unlock_tasks.into_iter() {
            //     unlock_futures.push();
            // }
            // if !unlock_futures.is_empty() {
            //     let now = Instant::now();
            //     let unlock_count = unlock_futures.len();
            //     join_all(unlock_futures).await;
            //     log::info!("unlock {} txs elapsed {:?}", unlock_count, now.elapsed());
            // }
        }
        Ok(())
    }
}

pub async fn get_ckb_tx_record(pool: &MySqlPool) -> Result<Vec<CkbToEthRecord>> {
    Ok(sqlx::query_as::<_, CkbToEthRecord>(
        r#"
SELECT *
FROM ckb_to_eth
        "#,
    )
    .fetch_all(pool)
    .await?)
}
