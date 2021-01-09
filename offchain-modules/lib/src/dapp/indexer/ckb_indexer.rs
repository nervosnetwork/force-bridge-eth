use crate::dapp::indexer::db::{
    create_ckb_to_eth_record, get_eth_to_ckb_record_by_outpoint, get_latest_ckb_to_eth_record,
    is_ckb_to_eth_record_exist, update_eth_to_ckb_status, CkbToEthRecord,
};
use crate::transfer::to_eth::parse_ckb_proof;
use crate::util::ckb_util::{create_bridge_lockscript, parse_cell};
use crate::util::config::{DeployedContracts, ForceConfig};
use crate::util::eth_util::convert_eth_address;
use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::Uint128;
use ckb_sdk::rpc::Transaction;
use ckb_sdk::HttpRpcClient;
use ckb_types::packed;
use ckb_types::packed::{Byte32, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use force_eth_types::eth_recipient_cell::ETHRecipientDataView;
use force_eth_types::generated::basic::ETHAddress;
use force_sdk::indexer::IndexerRpcClient;
use log::info;
use shellexpand::tilde;
use sqlx::MySqlPool;
use web3::types::H160;

pub struct CkbIndexer {
    // pub force_config: ForceConfig,
    pub config_path: String,
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
        // let force_config = ForceConfig::new(config_path.as_str())?;
        let rpc_client = HttpRpcClient::new(rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let db = MySqlPool::connect(&db_path).await?;
        Ok(CkbIndexer {
            config_path,
            rpc_client,
            indexer_client,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let record_option = get_latest_ckb_to_eth_record(&self.db).await?;
        let mut ckb_height;
        if record_option.is_some() {
            ckb_height = record_option.unwrap().block_number;
        } else {
            ckb_height = self
                .indexer_client
                .get_tip()
                .unwrap()
                .unwrap()
                .block_number
                .value();
        }
        loop {
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
                        self.handle_tx(tx.clone(), tx_hash, ckb_height).await?;
                    }
                }
                ckb_height += 1;
            } else {
                info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
            }
        }
    }

    pub async fn handle_tx(
        &mut self,
        tx: Transaction,
        tx_hash: String,
        block_number: u64,
    ) -> Result<()> {
        let ret = self
            .handle_burn_tx(tx.clone(), tx_hash, block_number)
            .await?;
        if !ret {
            self.handle_mint_tx(tx).await?;
        }
        Ok(())
    }

    pub async fn handle_burn_tx(
        &mut self,
        tx: Transaction,
        hash: String,
        block_number: u64,
    ) -> Result<bool> {
        if tx.outputs_data.is_empty() {
            return Ok(false);
        }
        let output_data = tx.outputs_data[0].as_bytes();
        let ret = ETHRecipientDataView::new(&output_data);
        if let Ok(eth_recipient) = ret {
            let force_config = ForceConfig::new(self.config_path.as_str())?;
            let deployed_contracts = force_config.deployed_contracts.as_ref().unwrap();
            let ret = self.verify_eth_recipient_data(eth_recipient.clone(), deployed_contracts)?;
            if ret {
                let recipient_typescript_code_hash =
                    hex::decode(&deployed_contracts.recipient_typescript.code_hash)
                        .map_err(|err| anyhow!(err))?;
                let typescript = tx.outputs[0].type_.as_ref().unwrap();
                if typescript.code_hash.as_bytes().to_vec() == recipient_typescript_code_hash {
                    // the tx is burn tx.
                    let token_addr: ETHAddress =
                        eth_recipient.eth_token_address.get_address().into();
                    let recipient_addr: ETHAddress =
                        eth_recipient.eth_recipient_address.get_address().into();
                    let token_amount = eth_recipient.token_amount;
                    let ckb_tx_proof = parse_ckb_proof(
                        hash.clone().as_str(),
                        String::from(self.rpc_client.url()),
                    )?;
                    let proof_str = serde_json::to_string(&ckb_tx_proof)?;
                    let tx_raw: packed::Transaction = tx.into();
                    let mol_hex_tx = hex::encode(tx_raw.raw().as_slice());

                    let record = CkbToEthRecord {
                        ckb_burn_tx_hash: hash.clone(),
                        status: "pending".to_string(),
                        token_addr: Some(hex::encode(token_addr.raw_data().to_vec().as_slice())),
                        recipient_addr: Some(hex::encode(
                            recipient_addr.raw_data().to_vec().as_slice(),
                        )),
                        token_amount: Some(Uint128::from(token_amount).to_string()),
                        ckb_spv_proof: Some(proof_str),
                        block_number,
                        ckb_raw_tx: Some(mol_hex_tx),
                        ..Default::default()
                    };
                    create_ckb_to_eth_record(&self.db, &record).await?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn verify_eth_recipient_data(
        &mut self,
        eth_recipient: ETHRecipientDataView,
        deployed_contracts: &DeployedContracts,
    ) -> Result<bool> {
        let light_client_typescript_hash_left = eth_recipient.light_client_typescript_hash;
        let cell_script = parse_cell(
            &deployed_contracts
                .light_client_cell_script
                .cell_script
                .as_str(),
        )?;
        let mut light_client_typescript_hash = [0u8; 32];
        light_client_typescript_hash
            .copy_from_slice(cell_script.calc_script_hash().raw_data().as_ref());
        let eth_bridge_lock_hash_left = eth_recipient.eth_bridge_lock_hash;
        let mut eth_bridge_lock_code_hash = [0u8; 32];
        eth_bridge_lock_code_hash.copy_from_slice(
            &hex::decode(&deployed_contracts.bridge_lockscript.code_hash)
                .map_err(|err| anyhow!(err))?,
        );

        if (light_client_typescript_hash_left == light_client_typescript_hash)
            && (eth_bridge_lock_hash_left == eth_bridge_lock_code_hash)
        {
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn handle_mint_tx(&mut self, tx: Transaction) -> Result<()> {
        let input = tx.inputs[0].previous_output.clone();
        let outpoint = OutPoint::new_builder()
            .tx_hash(Byte32::from_slice(input.tx_hash.as_ref())?)
            .index(input.index.pack())
            .build();
        let outpoint_hex = hex::encode(outpoint.as_slice());
        let ret = get_eth_to_ckb_record_by_outpoint(&self.db, outpoint_hex).await?;
        if let Some(mut eth_to_ckb_record) = ret {
            // check the tx is mint tx.
            let token_address_str = eth_to_ckb_record
                .clone()
                .token_addr
                .ok_or_else(|| anyhow!("the token address is not exist"))?;
            let token_address = convert_eth_address(token_address_str.as_str())?;
            let ret = self.check_bridge_lockscript(tx.clone(), token_address);
            tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
            if let Ok(success) = ret {
                if success {
                    eth_to_ckb_record.status = String::from("success");
                    update_eth_to_ckb_status(&self.db, &eth_to_ckb_record).await?;
                    log::info!("succeed to update eth_to_ckb cross status.");
                }
            }
        }
        Ok(())
    }

    pub fn check_bridge_lockscript(&mut self, tx: Transaction, token: H160) -> Result<bool> {
        let force_config = ForceConfig::new(self.config_path.as_str())?;
        let deployed_contracts = force_config.deployed_contracts.as_ref().unwrap();
        let sudt_script_json = tx.outputs[0]
            .clone()
            .type_
            .ok_or_else(|| anyhow!("the typescript is not exist"))?;
        let sudt_typescript_code_hash = hex::decode(&deployed_contracts.sudt.code_hash)?;
        let sudt_script_packed = packed::Script::from(sudt_script_json);

        let eth_address_str = &deployed_contracts.eth_token_locker_addr;
        let eth_address = convert_eth_address(eth_address_str.as_str())?;
        let lockscript = create_bridge_lockscript(deployed_contracts, &token, &eth_address)?;
        let code_hash = Byte32::from_slice(&sudt_typescript_code_hash)?;
        let sudt_typescript = Script::new_builder()
            .code_hash(code_hash)
            .hash_type(deployed_contracts.sudt.hash_type.into())
            .args(lockscript.calc_script_hash().as_bytes().pack())
            .build();
        Ok(sudt_script_packed.as_slice() == sudt_typescript.as_slice())
    }
}
