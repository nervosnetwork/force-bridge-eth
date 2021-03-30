use crate::util::ckb_util::{parse_cell, parse_merkle_cell_data};
use crate::util::config::ForceConfig;
use crate::util::eth_util::Web3Client;
use crate::util::rocksdb::{open_rocksdb, RocksDBStore, SMT};
use anyhow::{anyhow, Result};
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::IndexerRpcClient;
use log::info;
use rocksdb::ops::{Get, Put};
use rocksdb::DB;
use shellexpand::tilde;
use sparse_merkle_tree::traits::Value;
use std::path::Path;
use std::sync::Arc;
use web3::types::U64;

const ROCKSDB_MERKLE_ROOT_KEY: &[u8] = b"merkle";

pub struct EthHeaderIndexer {
    pub config_path: String,
    pub eth_client: Web3Client,
    pub indexer_client: IndexerRpcClient,
    pub rocksdb_path: String,
}

impl EthHeaderIndexer {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        rocksdb_path: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let indexer_client = IndexerRpcClient::new(ckb_indexer_url);

        Ok(EthHeaderIndexer {
            config_path,
            eth_client,
            indexer_client,
            rocksdb_path,
        })
    }

    pub async fn get_light_client_info(&mut self) -> Result<(u64, u64, [u8; 32])> {
        let config_path = tilde(self.config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("the deployed_contracts is not init."))?;
        let light_client_cell_script = deployed_contracts
            .light_client_cell_script
            .cell_script
            .as_str();
        let cell_script = parse_cell(light_client_cell_script)?;
        let cell = get_live_cell_by_typescript(&mut self.indexer_client, cell_script)
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("the cell is not exist"))?;
        let ckb_cell_data = cell.output_data.as_bytes().to_vec();
        if !ckb_cell_data.is_empty() {
            let (start_height, latest_height, merkle_root) =
                parse_merkle_cell_data(ckb_cell_data.to_vec())?;
            log::info!(
                "get_light_client_height start_height: {:?}, latest_height: {:?}",
                start_height,
                latest_height
            );

            return Ok((start_height, latest_height, merkle_root));
        }
        anyhow::bail!("waiting for the block confirmed!")
    }

    pub fn get_merkle_root(&mut self, db: Arc<DB>) -> Result<[u8; 32]> {
        let db_merkle_option = db
            .get(ROCKSDB_MERKLE_ROOT_KEY.to_vec())
            .map_err(|err| anyhow!(err))?;
        let merkle_root = match db_merkle_option {
            Some(v) => {
                let mut db_root_raw = [0u8; 32];
                db_root_raw.copy_from_slice(v.as_ref());
                db_root_raw
            }
            None => [0u8; 32],
        };
        Ok(merkle_root)
    }

    pub async fn loop_relay_rocksdb(&mut self) -> Result<()> {
        let mut latest_submit_height = 0;

        let db = open_rocksdb(self.rocksdb_path.clone());
        let mut merkle_root = self.get_merkle_root(db.clone())?;
        loop {
            let (start_height, latest_height, _) = self.get_light_client_info().await?;

            if latest_height <= latest_submit_height {
                log::info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                continue;
            }
            let (new_latest_submit_height, new_merkle_root) = self
                .relay_rocksdb(db.clone(), start_height, latest_height, merkle_root)
                .await?;
            latest_submit_height = new_latest_submit_height;
            merkle_root = new_merkle_root;
        }
    }

    pub async fn relay_rocksdb(
        &mut self,
        db: Arc<DB>,
        start_height: u64,
        latest_height: u64,
        merkle_root: [u8; 32],
    ) -> Result<(u64, [u8; 32])> {
        let eth_rocksdb_path = self.rocksdb_path.clone();
        let db_dir = tilde(eth_rocksdb_path.as_str()).into_owned();
        let db_path = Path::new(db_dir.as_str());
        let mut smt_tree = match db_path.exists() {
            false => {
                let rocksdb_store = RocksDBStore::new(eth_rocksdb_path.clone());
                SMT::new(sparse_merkle_tree::H256::zero(), rocksdb_store)
            }
            true => {
                let rocksdb_store = RocksDBStore::open(eth_rocksdb_path.clone());
                SMT::new(merkle_root.into(), rocksdb_store)
            }
        };

        let mut number = latest_height;
        while number >= start_height {
            let block_number = U64([number]);

            let mut key = [0u8; 32];
            let mut height = [0u8; 8];
            height.copy_from_slice(number.to_le_bytes().as_ref());
            key[..8].clone_from_slice(&height);

            let chain_block = self.eth_client.get_block(block_number.into()).await?;
            let chain_block_hash = chain_block
                .hash
                .ok_or_else(|| anyhow!("block hash should not be none"))?;

            let db_block_hash = smt_tree
                .get(&key.into())
                .map_err(|e| anyhow!("should get db block hash {}", e))?;

            info!(
                "chain_block_hash {:?}, db_block_hash {:?} height {}",
                chain_block_hash.0.as_slice(),
                db_block_hash.to_h256().as_slice(),
                number
            );

            if chain_block_hash.0.as_slice() != db_block_hash.to_h256().as_slice() {
                smt_tree
                    .update(key.into(), chain_block_hash.0.into())
                    .map_err(|e| anyhow!("should update {}", e))?;
                info!("Successfully relayed header {}", number);
            } else {
                break;
            }
            number -= 1;
        }

        let rocksdb_store = smt_tree.store_mut();
        rocksdb_store.commit()?;
        info!(
            "Successfully relayed the headers from {} to {}",
            number + 1,
            latest_height
        );

        let merkle_root = smt_tree.root();
        db.put(
            ROCKSDB_MERKLE_ROOT_KEY.to_vec(),
            merkle_root.as_slice().to_vec(),
        )
        .map_err(|err| anyhow!(err))?;

        Ok((latest_height, (*merkle_root).into()))
    }
}
