use crate::header_relay::ckb_relay::CKBRelayer;
use crate::util::ckb_tx_generator::Generator;
use crate::util::config::ForceConfig;
use crate::util::eth_util::Web3Client;
use crate::util::rocksdb::open_rocksdb;
use anyhow::{anyhow, Result};
use ckb_sdk::HttpRpcClient;
use force_sdk::indexer::IndexerRpcClient;
use rocksdb::ops::{Get, Put};
use rocksdb::DB;
use shellexpand::tilde;
use std::sync::Arc;

pub struct CkbHeaderIndexer {
    pub config_path: String,
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    pub eth_client: Web3Client,
    pub ckb_init_height: u64,
    pub rocksdb_path: String,
    pub confirm: u64,
}

impl CkbHeaderIndexer {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        rocksdb_path: String,
        confirm: u64,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let rpc_client = HttpRpcClient::new(ckb_rpc_url.clone());
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let indexer_client = IndexerRpcClient::new(ckb_indexer_url.clone());
        let eth_client = Web3Client::new(eth_rpc_url);

        let mut ckb_client = Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default())
            .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;

        let ckb_init_height = CKBRelayer::get_ckb_contract_deloy_height(
            &mut ckb_client,
            deployed_contracts
                .recipient_typescript
                .outpoint
                .tx_hash
                .clone(),
        )?;

        Ok(CkbHeaderIndexer {
            config_path,
            rpc_client,
            indexer_client,
            eth_client,
            rocksdb_path,
            ckb_init_height,
            confirm,
        })
    }

    pub async fn loop_relay_rocksdb(&mut self) -> Result<()> {
        let mut latest_submit_height = 0;
        let db = open_rocksdb(self.rocksdb_path.clone())?;

        loop {
            let ckb_current_height = self
                .rpc_client
                .get_tip_block_number()
                .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;
            let latest_height = ckb_current_height - self.confirm;

            if latest_height <= latest_submit_height {
                log::info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                continue;
            }
            latest_submit_height = self.relay_rocksdb(db.clone(), latest_height).await?;
        }
    }

    pub async fn relay_rocksdb(&mut self, db: Arc<DB>, latest_height: u64) -> Result<u64> {
        let mut index = latest_height;
        while index >= self.ckb_init_height {
            match self
                .rpc_client
                .get_block_by_number(index)
                .map_err(|e| anyhow!("get_header_by_number err: {:?}", e))?
            {
                Some(block_view) => {
                    let header_view = block_view.header;
                    let chain_root = header_view.inner.transactions_root.0;
                    let db_root_option = db.get(index.to_le_bytes()).map_err(|err| anyhow!(err))?;
                    let db_root = match db_root_option {
                        Some(v) => {
                            let mut db_root_raw = [0u8; 32];
                            db_root_raw.copy_from_slice(v.as_ref());
                            db_root_raw
                        }
                        None => [0u8; 32],
                    };

                    log::info!(
                        "chain_root {:?}, db_root {:?}, height {}",
                        chain_root.to_vec(),
                        db_root,
                        index
                    );
                    if chain_root.to_vec() != db_root {
                        db.put(index.to_le_bytes(), chain_root.to_vec())
                            .map_err(|err| anyhow!(err))?;
                    } else {
                        break;
                    }
                    index -= 1;
                }
                None => {
                    log::error!(
                        "cannot get the block transactions root, block_number = {}",
                        index
                    );
                    return Err(anyhow!(
                        "cannot get the block transactions root, block_number = {}",
                        index,
                    ));
                }
            }
        }
        log::info!(
            "store ckb headers from {:?} to {:?}",
            index + 1,
            latest_height
        );
        Ok(latest_height)
    }
}
