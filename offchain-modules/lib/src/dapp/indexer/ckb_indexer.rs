use crate::dapp::indexer::db::is_ckb_to_eth_record_exist;
use crate::util::config::ForceConfig;
use anyhow::{anyhow, Result};
use ckb_sdk::rpc::Transaction;
use ckb_sdk::HttpRpcClient;
use ckb_types::packed::{Byte32, OutPoint};
use ckb_types::prelude::{Builder, Entity, Pack};
use force_eth_types::eth_recipient_cell::ETHRecipientDataView;
use force_sdk::indexer::IndexerRpcClient;
use log::info;
use shellexpand::tilde;
use sqlx::MySqlPool;

pub struct CkbIndexer {
    pub force_config: ForceConfig,
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    pub db: MySqlPool,
}

impl CkbIndexer {
    pub async fn new(
        config_path: String,
        db_path: String,
        rpc_url: String,
        indexer_url: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let rpc_client = HttpRpcClient::new(rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let db = MySqlPool::connect(&db_path).await?;
        Ok(CkbIndexer {
            force_config,
            rpc_client,
            indexer_client,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut ckb_height = self
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;

        loop {
            dbg!(ckb_height);
            let block = self
                .rpc_client
                .get_block_by_number(ckb_height)
                .map_err(|e| anyhow!("failed to get ckb block by hash : {}", e))?;
            if let Some(block) = block {
                let txs = block.transactions;
                for tx_view in txs {
                    let tx = tx_view.inner;
                    let tx_hash = hex::encode(tx_view.hash.as_bytes());
                    let exist = is_ckb_to_eth_record_exist(&self.db, tx_hash.as_str()).await?;
                    if !exist {
                        self.handle_burn_tx(tx.clone()).await?;
                    }
                }
                ckb_height = ckb_height + 1;
            } else {
                info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
            }
        }
    }

    pub async fn handle_burn_tx(&mut self, tx: Transaction) -> Result<()> {
        let ret = self.verify_burn_tx(tx.clone())?;
        if !ret {
            self.handle_mint_tx(tx).await?;
        }
        Ok(())
    }

    pub fn verify_burn_tx(&mut self, tx: Transaction) -> Result<bool> {
        let output_data = tx.outputs_data[0].as_bytes();
        let ret = ETHRecipientDataView::new(&output_data);
        if let Ok(eth_recipient) = ret {
            let ret = self.verify_eth_recipient_data(eth_recipient)?;
            if ret {
                let recipient_typescript_code_hash = hex::decode(
                    &self
                        .force_config
                        .deployed_contracts
                        .as_ref()
                        .unwrap()
                        .recipient_typescript
                        .code_hash,
                )
                .map_err(|err| anyhow!(err))?;
                let typescript = tx.outputs[0].type_.as_ref().unwrap();
                if typescript.code_hash.as_bytes().to_vec() == recipient_typescript_code_hash {
                    // the tx is burn tx.
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn verify_eth_recipient_data(
        &mut self,
        eth_recipient: ETHRecipientDataView,
    ) -> Result<bool> {
        let light_client_typescript_hash_left =
            hex::encode(eth_recipient.light_client_typescript_hash);
        let light_client_typescript_hash_right = &self
            .force_config
            .deployed_contracts
            .as_ref()
            .unwrap()
            .light_client_typescript
            .code_hash;
        let eth_bridge_lock_hash_left = hex::encode(eth_recipient.eth_bridge_lock_hash);
        let eth_bridge_lock_hash_right = &self
            .force_config
            .deployed_contracts
            .as_ref()
            .unwrap()
            .bridge_lockscript
            .code_hash;
        if (light_client_typescript_hash_left == (*light_client_typescript_hash_right))
            && (eth_bridge_lock_hash_left == (*eth_bridge_lock_hash_right))
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn handle_mint_tx(&mut self, tx: Transaction) -> Result<()> {
        let input = tx.inputs[0].previous_output.clone();
        let outpoint = OutPoint::new_builder()
            .tx_hash(Byte32::from_slice(input.tx_hash.as_ref().clone())?)
            .index(input.index.pack())
            .build();
        let outpoint_hex = hex::encode(outpoint.as_slice());
        dbg!(outpoint_hex);
        Ok(())
    }
}
