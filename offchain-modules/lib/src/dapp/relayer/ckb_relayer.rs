use crate::transfer::to_eth::unlock;
use crate::util::config::ForceConfig;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CkbToEthRecord {
    pub id: i64,
    pub ckb_burn_tx_hash: String,
    pub status: String,
    pub recipient_addr: Option<String>,
    pub token_addr: Option<String>,
    pub token_amount: Option<String>,
    pub fee: Option<String>,
    pub eth_tx_hash: Option<String>,
    pub err_msg: Option<String>,
    pub ckb_spv_proof: Option<Vec<u8>>,
    pub ckb_raw_tx: Option<Vec<u8>>,
}

pub struct CkbTxRelay {
    config_path: String,
    eth_token_locker_addr: String,
    network: Option<String>,
    ethereum_rpc_url: String,
}

pub struct UnlockTask {
    pub burn_tx_proof: String,
    pub burn_raw_tx: String,
}

impl CkbTxRelay {
    pub fn new(config_path: String, network: Option<String>) -> Result<CkbTxRelay> {
        let force_config = ForceConfig::new(config_path.as_str())?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;
        let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;

        Ok(CkbTxRelay {
            config_path,
            eth_token_locker_addr: deployed_contracts.eth_token_locker_addr.clone(),
            network,
            ethereum_rpc_url,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let wait_relay_txs: Vec<CkbToEthRecord> = vec![];

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

            let mut unlock_futures = vec![];
            for task in unlock_tasks.into_iter() {
                unlock_futures.push(unlock(
                    self.config_path.clone(),
                    self.network.clone(),
                    "1".to_string(),
                    self.eth_token_locker_addr.clone(),
                    task.burn_tx_proof,
                    task.burn_raw_tx,
                    0,
                    true,
                ));
            }
            if !unlock_futures.is_empty() {
                let now = Instant::now();
                let unlock_count = unlock_futures.len();
                join_all(unlock_futures).await;
                log::info!("unlock {} txs elapsed {:?}", unlock_count, now.elapsed());
            }
        }
        Ok(())
    }
}
