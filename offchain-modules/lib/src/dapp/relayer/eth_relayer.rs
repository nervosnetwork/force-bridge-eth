use crate::transfer::to_ckb::send_eth_spv_proof_tx;
use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{get_eth_client_best_number, parse_privkey_path, ETHSPVProofJson};
use crate::util::config::ForceConfig;
use anyhow::{anyhow, Result};
use ckb_types::H256;
use crossbeam_channel::{bounded, Receiver, Sender};
use force_sdk::util::ensure_indexer_sync;
use futures::future::join_all;
use std::collections::hash_set::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct EthTxRelayer {
    pub config_path: String,
    pub force_config: ForceConfig,
    pub network: Option<String>,
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

pub struct MintTask {
    pub lock_tx_hash: String,
    pub lock_tx_proof: ETHSPVProofJson,
}

impl EthTxRelayer {
    pub fn new(
        config_path: String,
        network: Option<String>,
        privkey_index: String,
        db_args: String,
    ) -> Result<Self> {
        let force_config = ForceConfig::new(config_path.as_str())?;
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
            config_path,
            force_config,
            network,
            ckb_rpc_url,
            ckb_indexer_url,
            privkey_channel: PrivkeyChannel { sender, receiver },
            db_args,
            relaying_tx: Default::default(),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut latest_relayed_number = self.latest_relayed_number().await?;
        loop {
            latest_relayed_number = self.relay(latest_relayed_number).await?;
            tokio::time::delay_for(Duration::from_secs(10)).await
        }
    }

    async fn relay(&self, latest_relayed_number: u64) -> Result<u64> {
        let client_tip_number = self.client_tip_number().await?;
        log::info!(
            "start relay: last relayed number: {}, client tip number: {}",
            latest_relayed_number,
            client_tip_number
        );
        let mut mint_tasks = self
            .get_mint_tasks(latest_relayed_number, client_tip_number)
            .await?;

        // TODO write mint_tasks to eth_relay_record table
        // TODO write client_tip_number to db

        let retry_tasks = self.get_retry_tasks().await?;
        mint_tasks.extend(retry_tasks);
        let mut mint_futures = vec![];
        for task in mint_tasks.into_iter() {
            mint_futures.push(self.mint(task));
        }
        if !mint_futures.is_empty() {
            let now = Instant::now();
            let mint_count = mint_futures.len();
            join_all(mint_futures).await;
            log::info!("mint {} txs elapsed {:?}", mint_count, now.elapsed());
        }
        Ok(client_tip_number)
    }

    async fn latest_relayed_number(&self) -> Result<u64> {
        unimplemented!()
    }

    async fn client_tip_number(&self) -> Result<u64> {
        let mut generator = self.get_generator().await?;
        let force_contracts = self
            .force_config
            .deployed_contracts
            .clone()
            .expect("force contracts deployed");
        get_eth_client_best_number(
            &mut generator,
            force_contracts.light_client_cell_script.cell_script,
        )
    }

    async fn get_mint_tasks(
        &self,
        _from_block_number: u64,
        _to_block_number: u64,
    ) -> Result<Vec<MintTask>> {
        unimplemented!()
    }

    async fn get_retry_tasks(&self) -> Result<Vec<MintTask>> {
        unimplemented!()
    }

    async fn mint(&self, task: MintTask) {
        if let Err(error) = self.try_mint(task).await {
            if error.to_string().contains("irreparable error") {
                // TODO update db with error
            } else {
                // TODO update db with retryable and retry times
            }
        } else {
            // TODO delete db record
        }
    }

    async fn try_mint(&self, task: MintTask) -> Result<H256> {
        let mut generator = self.get_generator().await?;
        let mint_privkey = self.privkey_channel.get_privkey()?;
        let mint_privkey =
            parse_privkey_path(mint_privkey.as_str(), &self.force_config, &self.network)
                .expect("get ckb key succeed");
        send_eth_spv_proof_tx(
            &mut generator,
            self.config_path.clone(),
            task.lock_tx_hash,
            &task.lock_tx_proof,
            mint_privkey,
        )
        .await
    }

    async fn get_generator(&self) -> Result<Generator> {
        let force_contracts = self
            .force_config
            .deployed_contracts
            .clone()
            .expect("force contracts deployed");
        let mut generator = Generator::new(
            self.ckb_rpc_url.clone(),
            self.ckb_indexer_url.clone(),
            force_contracts,
        )
        .map_err(|e| anyhow!("new geneartor fail, err: {}", e))?;
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
            .await
            .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
        Ok(generator)
    }
}

impl PrivkeyChannel {
    pub fn get_privkey(&self) -> Result<String> {
        self.receiver
            .clone()
            .recv_timeout(Duration::from_secs(600))
            .map_err(|e| anyhow!("crossbeam channel recv ckb key timeout: {:?}", e))
    }
}
