use crate::util::config::ForceConfig;
use anyhow::Result;
use std::collections::hash_set::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use futures::future::join_all;
use crossbeam_channel::{bounded, Receiver, Sender};

mod error;

pub struct EthTxRelayer {
    pub force_config: ForceConfig,
    pub ckb_rpc_url: String,
    pub ckb_indexer_url: String,
    pub privkey_channel: PrivkeyChannel,
    pub db_args: String,
    pub relaying_tx: Arc<Mutex<HashSet<String>>>,
}

pub struct PrivkeyChannel {
    pub sender: Sender<String>,
    pub receiver: Receiver<String>,
}

struct MintTask {}

impl EthTxRelayer {
    pub fn new(config_path: &str, network: Option<String>, privkey_index: String, db_args: String) -> Result<Self> {
        let force_config = ForceConfig::new(config_path)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let privkey_index = privkey_index.as_str().parse::<usize>()?;
        let privkey_length = force_config.get_ckb_private_keys(&network)?.len();
        assert!(
            privkey_length > privkey_index,
            "invalid args: ckb_private_key_path"
        );
        let (sender, receiver) = bounded(privkey_length - privkey_index);
        for i in privkey_index..privkey_length {
            sender
                .send(i.to_string())
                .expect("init ckb private key channel succeed");
        }
        Ok(EthTxRelayer {
            force_config,
            ckb_rpc_url,
            ckb_indexer_url,
            privkey_channel: PrivkeyChannel{sender, receiver},
            db_args,
            relaying_tx: Default::default()
        })
    }

    pub async fn start(&self) -> Result<()> {
        // TODO if the eth_relay_tx table is empty, insert first record with minimum block number in eth_to_ckb_record table
        let mut latest_relayed_number = self.latest_relayed_number().await?;
        loop {
            latest_relayed_number = self.relay(latest_relayed_number).await?;
            tokio::time::delay_for(Duration::from_secs(60)).await
        }
    }

    async fn relay(&self, latest_relayed_number: u64) -> Result<u64> {
        let client_tip_number = self.client_tip_number().await?;
        let mut mint_tasks = self.get_mint_tasks(latest_relayed_number, client_tip_number).await?;

        // TODO write mint_tasks to eth_relay_record table

        let retry_tasks = self.get_retry_tasks().await?;
        mint_tasks.extend(retry_tasks);
        let mut mint_futures = vec![];
        for task in mint_tasks.into_iter() {
            mint_futures.push(self.mint(task));
        }
        if !mint_futures.is_empty() {
            join_all(mint_futures).await;
        }

        // TODO write client_tip_number to db

        Ok(client_tip_number)
    }

    async fn latest_relayed_number(&self) -> Result<u64> {
        unimplemented!()
    }

    async fn client_tip_number(&self) -> Result<u64> {
        unimplemented!()
    }

    async fn get_mint_tasks(&self, from_block_number: u64, to_block_number: u64) -> Result<Vec<MintTask>> {
        unimplemented!()
    }

    async fn get_retry_tasks(&self) -> Result<Vec<MintTask>> {
        unimplemented!()
    }

    async fn mint(&self, task: MintTask) -> Result<()> {
        unimplemented!()
    }
}
