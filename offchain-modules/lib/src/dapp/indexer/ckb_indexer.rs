use crate::dapp::db::indexer::{
    create_ckb_to_eth_record, delete_ckb_to_eth_records, delete_ckb_unconfirmed_block,
    get_ckb_unconfirmed_block, get_ckb_unconfirmed_blocks, get_eth_to_ckb_record_by_outpoint,
    get_height_info, get_max_ckb_unconfirmed_block, insert_ckb_unconfirmed_block,
    insert_ckb_unconfirmed_blocks, is_ckb_to_eth_record_exist, reset_eth_to_ckb_record_status,
    update_ckb_unconfirmed_block, update_cross_chain_height_info, update_eth_to_ckb_status,
    CkbToEthRecord, CkbUnConfirmedBlock, CrossChainHeightInfo, EthToCkbRecord,
};
use crate::util::ckb_util::{clear_0x, create_bridge_lockscript, parse_cell};
use crate::util::config::{DeployedContracts, ForceConfig};
use crate::util::eth_util::{convert_eth_address, Web3Client};
use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::Uint128;
use ckb_sdk::rpc::{BlockView, Transaction};
use ckb_sdk::HttpRpcClient;
use ckb_types::packed;
use ckb_types::packed::{Byte32, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use force_eth_types::eth_recipient_cell::ETHRecipientDataView;
use force_eth_types::generated::basic::ETHAddress;
use force_sdk::indexer::IndexerRpcClient;
use shellexpand::tilde;
use sqlx::MySqlPool;
use web3::types::H160;

pub const CKB_CHAIN_CONFIRMED: usize = 15;

pub struct CkbIndexer {
    // pub force_config: ForceConfig,
    pub config_path: String,
    pub rpc_client: HttpRpcClient,
    pub indexer_client: IndexerRpcClient,
    pub eth_client: Web3Client,
    pub db: MySqlPool,
}

impl CkbIndexer {
    pub async fn new(
        config_path: String,
        db_path: String,
        network: Option<String>,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let rpc_client = HttpRpcClient::new(ckb_rpc_url);
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let indexer_client = IndexerRpcClient::new(ckb_indexer_url);
        let db = MySqlPool::connect(&db_path).await?;
        let eth_client = Web3Client::new(eth_rpc_url);
        Ok(CkbIndexer {
            config_path,
            rpc_client,
            indexer_client,
            eth_client,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let force_config = ForceConfig::new(self.config_path.as_str())?;
        let eth_contract_addr = force_config
            .deployed_contracts
            .ok_or_else(|| anyhow!("the deployed_contracts is not exist"))?
            .eth_ckb_chain_addr;
        let contract_addr = convert_eth_address(&eth_contract_addr)?;
        let mut height_info = get_height_info(&self.db, 2 as u8).await?;
        if height_info.height == 0 {
            // height info init.
            height_info.height = self
                .eth_client
                .get_contract_height("latestBlockNumber", contract_addr)
                .await?;
        }
        let mut start_block_number = height_info.height + 1;
        let mut unconfirmed_blocks = get_ckb_unconfirmed_blocks(&self.db).await?;
        if unconfirmed_blocks.is_empty() {
            // init unconfirmed_blocks
            self.init_ckb_unconfirmed_blocks(&mut unconfirmed_blocks, start_block_number)
                .await?;
        }
        let mut tail = get_max_ckb_unconfirmed_block(&self.db)
            .await?
            .ok_or_else(|| anyhow!("the tail is not exist"))?;
        if unconfirmed_blocks[unconfirmed_blocks.len() - 1].hash != tail.hash
            || tail.number + 1 != start_block_number
        {
            anyhow::bail!("system error! the unconfirmed_blocks is invalid.")
        }
        let mut re_org = false;
        loop {
            log::info!("handle ckb block number: {:?}", start_block_number);
            let block = self
                .rpc_client
                .get_block_by_number(start_block_number)
                .map_err(|e| anyhow!("failed to get ckb block by hash : {}", e))?;
            if block.is_none() {
                log::info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                continue;
            }
            let mut block = block.unwrap();
            if hex::encode(block.header.inner.parent_hash.clone()) != tail.hash {
                // the chain is re_organized.
                log::info!("the chain is re_organized");
                re_org = true;
                block = self
                    .lookup_common_ancestor_in_ckb(
                        &unconfirmed_blocks,
                        (unconfirmed_blocks.len() - 1) as isize,
                    )
                    .await?;
                log::info!("find the common ancestor ckb block: {:?}", block);
                start_block_number = block.header.inner.number + 1;
                unconfirmed_blocks = unconfirmed_blocks
                    .into_iter()
                    .filter(|s| s.number < start_block_number)
                    .collect();
                block = self
                    .rpc_client
                    .get_block_by_number(start_block_number)
                    .map_err(|e| anyhow!("failed to get ckb block by hash : {}", e))?
                    .ok_or_else(|| anyhow!("the block is not exist."))?;
            }
            self.handle_ckb_event(
                &block,
                contract_addr,
                &mut start_block_number,
                &mut unconfirmed_blocks,
                &mut tail,
                re_org,
            )
            .await?;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn handle_ckb_event(
        &mut self,
        block: &BlockView,
        contract_addr: H160,
        start_block_number: &mut u64,
        unconfirmed_blocks: &mut Vec<CkbUnConfirmedBlock>,
        tail: &mut CkbUnConfirmedBlock,
        re_org: bool,
    ) -> Result<()> {
        let txs = block.transactions.clone();
        let mut burn_records = vec![];
        let mut mint_records = vec![];
        for tx_view in txs {
            let tx = tx_view.inner;
            let tx_hash = hex::encode(tx_view.hash.as_bytes());
            let exist = is_ckb_to_eth_record_exist(&self.db, tx_hash.as_str()).await?;
            if !exist {
                let is_burn_tx = self
                    .handle_burn_tx(
                        tx.clone(),
                        tx_hash.clone(),
                        *start_block_number,
                        &mut burn_records,
                    )
                    .await?;
                if !is_burn_tx {
                    self.handle_mint_tx(tx, tx_hash, &mut mint_records, *start_block_number)
                        .await?;
                }
            }
        }
        let height_info = CrossChainHeightInfo {
            id: 2,
            height: *start_block_number,
            client_height: self
                .eth_client
                .get_contract_height("latestBlockNumber", contract_addr)
                .await?,
        };
        let mut unconfirmed_block;
        let hash_str = hex::encode(block.header.hash.clone());
        if unconfirmed_blocks.len() < CKB_CHAIN_CONFIRMED {
            unconfirmed_block = CkbUnConfirmedBlock {
                id: *start_block_number % CKB_CHAIN_CONFIRMED as u64,
                number: *start_block_number,
                hash: hash_str,
            };
        } else {
            unconfirmed_block = get_ckb_unconfirmed_block(
                &self.db,
                *start_block_number % CKB_CHAIN_CONFIRMED as u64,
            )
            .await?
            .ok_or_else(|| anyhow!("the block is not exist"))?;
            unconfirmed_block.number = *start_block_number;
            unconfirmed_block.hash = hash_str;
        }
        let mut db_tx = self.db.begin().await?;
        if re_org {
            // delete eth to ckb record while the block number > start_block_number
            delete_ckb_to_eth_records(&mut db_tx, *start_block_number).await?;
            reset_eth_to_ckb_record_status(&mut db_tx, *start_block_number).await?;
            delete_ckb_unconfirmed_block(&mut db_tx, *start_block_number).await?;
        }
        if !burn_records.is_empty() {
            create_ckb_to_eth_record(&mut db_tx, &burn_records).await?;
        }
        if !mint_records.is_empty() {
            for item in mint_records {
                update_eth_to_ckb_status(&mut db_tx, &item).await?;
            }
        }
        update_cross_chain_height_info(&mut db_tx, &height_info).await?;
        if unconfirmed_blocks.len() < CKB_CHAIN_CONFIRMED {
            insert_ckb_unconfirmed_block(&mut db_tx, &unconfirmed_block).await?
        } else {
            update_ckb_unconfirmed_block(&mut db_tx, &unconfirmed_block).await?;
            unconfirmed_blocks.remove(0);
        }
        db_tx.commit().await?;
        *start_block_number += 1;
        *tail = unconfirmed_block.clone();
        unconfirmed_blocks.push(unconfirmed_block);
        Ok(())
    }

    pub async fn init_ckb_unconfirmed_blocks(
        &mut self,
        unconfirmed_blocks: &mut Vec<CkbUnConfirmedBlock>,
        start_block_number: u64,
    ) -> Result<()> {
        let mut start = 0;
        if start_block_number > CKB_CHAIN_CONFIRMED as u64 {
            start = start_block_number - CKB_CHAIN_CONFIRMED as u64;
        }
        for i in start..start_block_number {
            let block = self
                .rpc_client
                .get_block_by_number(i)
                .map_err(|e| anyhow!("failed to get ckb block by hash : {}", e))?
                .ok_or_else(|| anyhow!("the block is not exist"))?;
            let number = block.header.inner.number;
            let record = CkbUnConfirmedBlock {
                id: number % CKB_CHAIN_CONFIRMED as u64,
                number,
                hash: hex::encode(block.header.hash),
            };
            unconfirmed_blocks.push(record);
        }
        insert_ckb_unconfirmed_blocks(&self.db, &unconfirmed_blocks).await?;
        Ok(())
    }

    pub async fn lookup_common_ancestor_in_ckb(
        &mut self,
        blocks: &[CkbUnConfirmedBlock],
        mut index: isize,
    ) -> Result<BlockView> {
        while index >= 0 {
            let latest_block = &blocks[index as usize];
            let block = self
                .rpc_client
                .get_block_by_number(latest_block.number)
                .map_err(|e| anyhow!("failed to get ckb block by hash : {}", e))?;
            if block.is_some() {
                let block = block.unwrap();
                if hex::encode(&block.header.hash) == latest_block.hash {
                    return Ok(block);
                }
            }
            // The latest header on ckb is not on the Ethereum main chain and needs to be backtracked
            index -= 1;
        }
        anyhow::bail!("system error! can not find the common ancestor with main chain.")
    }

    pub async fn handle_burn_tx(
        &mut self,
        tx: Transaction,
        hash: String,
        block_number: u64,
        burn_records: &mut Vec<CkbToEthRecord>,
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
                let locker_addr: ETHAddress =
                    eth_recipient.eth_lock_contract_address.get_address().into();
                let lock_contract_addr = hex::encode(locker_addr.raw_data().to_vec().as_slice());
                if typescript.code_hash.as_bytes().to_vec() == recipient_typescript_code_hash
                    && lock_contract_addr.to_lowercase().as_str()
                        == clear_0x(
                            deployed_contracts
                                .eth_token_locker_addr
                                .to_lowercase()
                                .as_str(),
                        )
                {
                    // the tx is burn tx.
                    // let locker_addr: ETHAddress =
                    //     eth_recipient.eth_lock_contract_address.get_address().into();
                    let token_addr: ETHAddress =
                        eth_recipient.eth_token_address.get_address().into();
                    let recipient_addr: ETHAddress =
                        eth_recipient.eth_recipient_address.get_address().into();
                    let token_amount = eth_recipient.token_amount;
                    // let ckb_unlock_token_param = parse_ckb_proof(
                    //     hash.as_str(),
                    //     String::from(self.rpc_client.url()),
                    //     String::from(self.eth_client.url()),
                    //     H160::from_slice(locker_addr.as_slice()),
                    // )
                    // .await?;
                    // let ckb_history_tx_proof: ckb_tx_proof::CKBHistoryTxProof =
                    //     ckb_unlock_token_param.tx_proofs[0].clone().into();

                    // let proof_str = hex::encode(ckb_history_tx_proof.as_bytes().as_ref());
                    // let tx_raw: packed::Transaction = tx.into();
                    // let mol_hex_tx = hex::encode(tx_raw.raw().as_slice());

                    let record = CkbToEthRecord {
                        ckb_burn_tx_hash: hash,
                        status: "pending".to_string(),
                        token_addr: hex::encode(token_addr.raw_data().to_vec().as_slice()),
                        recipient_addr: hex::encode(recipient_addr.raw_data().to_vec().as_slice()),
                        token_amount: Uint128::from(token_amount).to_string(),
                        // ckb_spv_proof: Some(proof_str),
                        ckb_block_number: block_number,
                        // ckb_raw_tx: mol_hex_tx,
                        fee: Uint128::from(eth_recipient.fee).to_string(),
                        bridge_lock_hash: hex::encode(
                            eth_recipient.eth_bridge_lock_hash.as_slice(),
                        ),
                        lock_contract_addr,
                        ..Default::default()
                    };
                    burn_records.push(record);
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

    pub async fn handle_mint_tx(
        &mut self,
        tx: Transaction,
        tx_hash_str: String,
        unlock_datas: &mut Vec<EthToCkbRecord>,
        number: u64,
    ) -> Result<()> {
        let force_config = ForceConfig::new(self.config_path.as_str())?;
        let deployed_contracts = force_config.deployed_contracts.as_ref().unwrap();
        let input = tx.inputs[0].previous_output.clone();
        let outpoint = OutPoint::new_builder()
            .tx_hash(Byte32::from_slice(input.tx_hash.as_ref())?)
            .index(input.index.pack())
            .build();
        let outpoint_hex = hex::encode(outpoint.as_slice());
        if !tx.outputs.is_empty() {
            let sudt_script_json = tx.outputs[0].clone().type_;
            if let Some(type_script) = sudt_script_json {
                let sudt_script = packed::Script::from(type_script);
                let ret = is_mint_tx(tx.clone(), &deployed_contracts, sudt_script.clone());
                if let Ok(mint_tx) = ret {
                    if mint_tx {
                        let ret = get_eth_to_ckb_record_by_outpoint(&self.db, outpoint_hex).await?;
                        if let Some(mut eth_to_ckb_record) = ret {
                            // check the tx is mint tx.
                            let token_address_str = eth_to_ckb_record.clone().token_addr;
                            let token_address = convert_eth_address(token_address_str.as_str())?;
                            let ret = self.check_bridge_lockscript(
                                token_address,
                                &deployed_contracts,
                                sudt_script,
                            );
                            if let Ok(success) = ret {
                                if success {
                                    eth_to_ckb_record.status = String::from("success");
                                    eth_to_ckb_record.ckb_block_number = number;
                                    eth_to_ckb_record.ckb_tx_hash = Some(tx_hash_str);
                                    unlock_datas.push(eth_to_ckb_record);
                                }
                            }
                            return Ok(());
                        }
                        log::info!(
                            "the lock tx is not exist. waiting for eth indexer reach sync status."
                        );
                        tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                        anyhow::bail!(
                            "the lock tx is not exist. waiting for eth indexer reach sync status."
                        )
                    }
                }
            }
        }

        Ok(())
    }

    pub fn check_bridge_lockscript(
        &mut self,
        token: H160,
        deployed_contracts: &DeployedContracts,
        sudt_script_packed: Script,
    ) -> Result<bool> {
        // let sudt_script_json = tx.outputs[0]
        //     .clone()
        //     .type_
        //     .ok_or_else(|| anyhow!("the typescript is not exist"))?;
        let sudt_typescript_code_hash = hex::decode(&deployed_contracts.sudt.code_hash)?;
        // let sudt_script_packed = packed::Script::from(sudt_script_json);

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

pub fn is_mint_tx(
    tx: Transaction,
    deployed_contracts: &DeployedContracts,
    sudt_script: Script,
) -> Result<bool> {
    let sudt_typescript_code_hash = hex::decode(&deployed_contracts.sudt.code_hash)?;
    let bridge_typescript_code_hash = hex::decode(&deployed_contracts.bridge_typescript.code_hash)?;
    let simple_typescript_code_hash =
        hex::decode(&deployed_contracts.simple_bridge_typescript.code_hash)?;
    if sudt_script.code_hash().as_slice() == sudt_typescript_code_hash.as_slice() {
        for i in 1..tx.outputs.len() {
            let bridge_type_script_op = tx.outputs[i].clone().type_;
            if let Some(bridge_type_script) = bridge_type_script_op {
                if bridge_type_script.code_hash.as_bytes() == bridge_typescript_code_hash.as_slice()
                    || bridge_type_script.code_hash.as_bytes()
                        == simple_typescript_code_hash.as_slice()
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
