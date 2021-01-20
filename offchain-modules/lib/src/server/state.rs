use crate::util::ckb_tx_generator::Generator;
use crate::util::config::{DeployedContracts, ForceConfig};
use crate::util::eth_util::Web3Client;
use anyhow::{anyhow, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use force_sdk::util::ensure_indexer_sync;
use shellexpand::tilde;
use sqlx::mysql::MySqlPool;
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct DappState {
    pub config_path: String,
    pub network: Option<String>,
    pub ckb_key_channel: (Sender<String>, Receiver<String>),
    pub eth_key_channel: (Sender<String>, Receiver<String>),
    pub deployed_contracts: DeployedContracts,
    pub indexer_url: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub db: MySqlPool,
    pub relaying_txs: Arc<Mutex<HashSet<String>>>,
}

impl DappState {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        ckb_private_key_path: String,
        eth_private_key_path: String,
        db_path: String,
        alarm_url: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let indexer_url = force_config.get_ckb_indexer_url(&network)?;

        let ckb_key_start_index = ckb_private_key_path.as_str().parse::<usize>()?;
        let ckb_key_len = force_config.get_ckb_private_keys(&network)?.len();
        assert!(
            ckb_key_len > ckb_key_start_index,
            "invalid args: ckb_private_key_path"
        );
        let (ckb_key_sender, ckb_key_receiver) = bounded(ckb_key_len - ckb_key_start_index);
        for i in ckb_key_start_index..ckb_key_len {
            ckb_key_sender
                .send(i.to_string())
                .expect("init ckb private key pool succeed");
        }

        let eth_key_start_index = eth_private_key_path.as_str().parse::<usize>()?;
        let eth_key_len = force_config.get_ethereum_private_keys(&network)?.len();
        assert!(
            eth_key_len > eth_key_start_index,
            "invalid args: eth_private_key_path"
        );
        let (eth_key_sender, eth_key_receiver) = bounded(eth_key_len - eth_key_start_index);
        for i in eth_key_start_index..eth_key_len {
            eth_key_sender
                .send(i.to_string())
                .expect("init eth private key pool succeed");
        }

        // let from_privkey =
        //     parse_privkey_path(ckb_private_key_path.as_str(), &force_config, &network)?;
        let db = MySqlPool::connect(&db_path).await?;
        let db2 = db.clone();
        tokio::spawn(db_monitor(db2, alarm_url));
        Ok(Self {
            ckb_key_channel: (ckb_key_sender, ckb_key_receiver),
            eth_key_channel: (eth_key_sender, eth_key_receiver),
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            deployed_contracts: force_config
                .deployed_contracts
                .expect("contracts should be deployed"),
            network,
            db,
            relaying_txs: Arc::new(Mutex::new(HashSet::default())),
        })
    }

    pub async fn get_generator(&self) -> Result<Generator> {
        let mut generator = Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.deployed_contracts.clone(),
        )
        .map_err(|e| anyhow!("new geneartor fail, err: {}", e))?;
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
            .await
            .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
        Ok(generator)
    }

    pub fn get_web3_client(&self) -> Web3Client {
        Web3Client::new(self.eth_rpc_url.clone())
    }

    pub async fn add_relaying_tx(&self, tx_hash: String) -> bool {
        let mut relaying_txs = self.relaying_txs.clone().lock_owned().await;
        return if relaying_txs.contains(&tx_hash) {
            false
        } else {
            relaying_txs.insert(tx_hash);
            true
        };
    }

    pub async fn remove_relaying_tx(&self, tx_hash: String) {
        let mut relaying_txs = self.relaying_txs.clone().lock_owned().await;
        relaying_txs.remove(&tx_hash);
    }
}

/// monitor db, send alarm when there are not successful records
async fn db_monitor(pool: MySqlPool, alarm_url: String) {
    loop {
        let res = db_monitor_inner(&pool, &alarm_url).await;
        if let Err(e) = res {
            log::error!("fail to check db monitor: {}", e);
        } else {
            log::info!("no failed records in db");
        }
        tokio::time::delay_for(std::time::Duration::from_secs(300)).await;
    }
}

async fn db_monitor_inner(pool: &MySqlPool, alarm_url: &str) -> Result<()> {
    let records = super::proof_relayer::db::get_eth_to_ckb_failed_records(&pool).await?;
    let counter = records.iter().fold(HashMap::new(), |mut acc, c| {
        *acc.entry(c.status.clone()).or_insert(0) += 1u64;
        acc
    });
    let msg = format!("db records stat: {:?}", &counter);
    log::info!(
        "db records stat: {:?}\nnot successful records: {:?}",
        &counter,
        &records
    );
    let res = reqwest::get(format!("{}{}", &alarm_url, msg).as_str())
        .await?
        .text()
        .await?;
    log::info!("{:?}", res);
    Ok(())
}
