use crate::dapp::db::indexer::{
    create_eth_to_ckb_record, delete_eth_to_ckb_records, delete_eth_unconfirmed_block,
    get_eth_unconfirmed_block, get_eth_unconfirmed_blocks, get_height_info,
    get_max_eth_unconfirmed_block, insert_eth_unconfirmed_block, insert_eth_unconfirmed_blocks,
    reset_ckb_to_eth_record_status, update_ckb_to_eth_record_status,
    update_cross_chain_height_info, update_eth_unconfirmed_block, CrossChainHeightInfo,
    EthToCkbRecord, EthUnConfirmedBlock,
};
use crate::dapp::indexer::IndexerFilter;
use crate::transfer::to_ckb::to_eth_spv_proof_json;
use crate::util::ckb_util::{clear_0x, parse_cell, parse_main_chain_headers};
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_hex_to_h256, Web3Client};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::Uint128;
use ethabi::{Function, Param, ParamType};
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::IndexerRpcClient;
use log::info;
use rusty_receipt_proof_maker::types::UnlockEvent;
use rusty_receipt_proof_maker::{
    generate_eth_proof, parse_event, parse_unlock_event, types::EthSpvProof,
};
use shellexpand::tilde;
use sqlx::MySqlPool;
use web3::types::{Block, H256, U64};

pub const ETH_CHAIN_CONFIRMED: usize = 15;

pub struct EthIndexer<T> {
    pub config_path: String,
    pub eth_client: Web3Client,
    pub db: MySqlPool,
    pub indexer_client: IndexerRpcClient,
    pub indexer_filter: T,
}

impl<T: IndexerFilter> EthIndexer<T> {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        db_path: String,
        indexer_url: String,
        indexer_filter: T,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let indexer_client = IndexerRpcClient::new(indexer_url);
        let db = MySqlPool::connect(&db_path).await?;
        Ok(EthIndexer {
            config_path,
            eth_client,
            db,
            indexer_client,
            indexer_filter,
        })
    }

    pub async fn get_light_client_height(&mut self) -> Result<u64> {
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
            let (un_confirmed_headers, _) = parse_main_chain_headers(ckb_cell_data)?;
            let best_block_height = un_confirmed_headers[un_confirmed_headers.len() - 1]
                .number
                .ok_or_else(|| anyhow!("the number is not exist"))?
                .as_u64();
            return Ok(best_block_height);
        }
        anyhow::bail!("waiting for the block confirmed!")
    }

    pub async fn get_light_client_height_with_loop(&mut self) -> u64 {
        loop {
            let ret = self.get_light_client_height().await;
            match ret {
                Ok(number) => {
                    return number;
                }
                Err(err) => {
                    log::error!("failed to get light client height.Err: {:?}", err);
                    tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    #[allow(unused_assignments)]
    pub async fn start(&mut self) -> Result<()> {
        let mut height_info = get_height_info(&self.db, 1 as u8).await?;
        if height_info.height == 0 {
            // height info init.
            height_info.height = self.get_light_client_height_with_loop().await;
        }
        let mut start_block_number = height_info.height + 1;
        let mut unconfirmed_blocks = get_eth_unconfirmed_blocks(&self.db).await?;
        if unconfirmed_blocks.is_empty() {
            // init unconfirmed_blocks
            let mut start = 0;
            if start_block_number > ETH_CHAIN_CONFIRMED as u64 {
                start = start_block_number - ETH_CHAIN_CONFIRMED as u64;
            }
            let blocks = self
                .eth_client
                .get_blocks(start, start_block_number)
                .await?;
            for item in blocks {
                let number = item
                    .number
                    .ok_or_else(|| anyhow!("the number is not exist."))?
                    .as_u64();
                let record = EthUnConfirmedBlock {
                    id: number % ETH_CHAIN_CONFIRMED as u64,
                    number,
                    hash: hex::encode(item.hash.ok_or_else(|| anyhow!("the hash is not exist."))?),
                };
                unconfirmed_blocks.push(record);
            }
            insert_eth_unconfirmed_blocks(&self.db, &unconfirmed_blocks).await?;
        }
        let config_path = tilde(self.config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let lock_contract_address = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("the deployed_contracts is not init"))?
            .eth_token_locker_addr
            .clone();
        let mut tail = get_max_eth_unconfirmed_block(&self.db)
            .await?
            .ok_or_else(|| anyhow!("the tail is not exist"))?;
        log::info!(
            "tail number: {:?}, start number: {:?}",
            tail.number,
            start_block_number
        );
        if unconfirmed_blocks[unconfirmed_blocks.len() - 1].hash != tail.hash
            || tail.number + 1 != start_block_number
        {
            anyhow::bail!("system error! the unconfirmed_blocks is invalid.")
        }
        let mut re_org = false;

        loop {
            log::info!("handle block number: {:?}", start_block_number);
            let block = self
                .eth_client
                .get_block(U64::from(start_block_number).into())
                .await;
            if block.is_err() {
                log::info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                continue;
            }
            let mut block = block.unwrap();
            let left = hex::encode(block.parent_hash);
            if left != tail.hash {
                // the chain is re_organized.
                log::info!("the chain is re_organized");
                re_org = true;
                block = self
                    .lookup_common_ancestor_in_eth(
                        &unconfirmed_blocks,
                        // (ETH_CHAIN_CONFIRMED - 1) as isize,
                        (unconfirmed_blocks.len() - 1) as isize,
                    )
                    .await?;
                start_block_number = block
                    .number
                    .ok_or_else(|| anyhow!("invalid block number"))?
                    .as_u64()
                    + 1;
                unconfirmed_blocks = unconfirmed_blocks
                    .into_iter()
                    .filter(|s| s.number < start_block_number)
                    .collect();
                block = self
                    .eth_client
                    .get_block(U64::from(start_block_number).into())
                    .await
                    .map_err(|err| anyhow!(err))?;
            }
            let mut lock_records = vec![];
            let mut unlock_records = vec![];
            let (lock_vec, unlock_vec) = self.parse_event_with_retry(
                hex::encode(block.hash.unwrap()),
                5,
                lock_contract_address.clone(),
            )?;
            if !lock_vec.is_empty() {
                for item in lock_vec {
                    self.handle_lock_event(
                        &mut lock_records,
                        lock_contract_address.clone(),
                        &item,
                        start_block_number,
                    )
                    .await?;
                }
            }
            if !unlock_vec.is_empty() {
                for item in unlock_vec {
                    self.handle_unlock_event(item.tx_hash, &mut unlock_records)
                        .await?;
                }
            }
            height_info = get_height_info(&self.db, 1 as u8).await?;
            let mut unconfirmed_block;
            let hash_str = hex::encode(
                block
                    .hash
                    .ok_or_else(|| anyhow!("the block is not exist"))?,
            );
            if unconfirmed_blocks.len() < ETH_CHAIN_CONFIRMED {
                unconfirmed_block = EthUnConfirmedBlock {
                    id: start_block_number % ETH_CHAIN_CONFIRMED as u64,
                    number: start_block_number,
                    hash: hash_str,
                };
            } else {
                unconfirmed_block = get_eth_unconfirmed_block(
                    &self.db,
                    start_block_number % ETH_CHAIN_CONFIRMED as u64,
                )
                .await?
                .ok_or_else(|| anyhow!("the block is not exist"))?;
                unconfirmed_block.number = start_block_number;
                unconfirmed_block.hash = hash_str;
            }
            let mut db_tx = self.db.begin().await?;
            if re_org {
                // delete eth to ckb record while the block number > start_block_number
                delete_eth_to_ckb_records(&mut db_tx, start_block_number).await?;
                reset_ckb_to_eth_record_status(&mut db_tx, start_block_number).await?;
                delete_eth_unconfirmed_block(&mut db_tx, start_block_number).await?;
            }
            if !lock_records.is_empty() {
                create_eth_to_ckb_record(&mut db_tx, &lock_records).await?;
            }
            if !unlock_records.is_empty() {
                for item in unlock_records {
                    update_ckb_to_eth_record_status(
                        &mut db_tx,
                        item.0,
                        item.1,
                        "success",
                        start_block_number,
                    )
                    .await?;
                }
            }
            let light_client_height = self.get_light_client_height_with_loop().await;
            // let eth_height = self.eth_client.client().eth().block_number().await?;
            let height_info = CrossChainHeightInfo {
                id: 1,
                height: start_block_number,
                client_height: light_client_height,
            };
            update_cross_chain_height_info(&mut db_tx, &height_info).await?;
            if unconfirmed_blocks.len() < ETH_CHAIN_CONFIRMED {
                insert_eth_unconfirmed_block(&mut db_tx, &unconfirmed_block).await?
            } else {
                update_eth_unconfirmed_block(&mut db_tx, &unconfirmed_block).await?;
                unconfirmed_blocks.remove(0);
            }
            db_tx.commit().await?;
            start_block_number += 1;
            tail = unconfirmed_block.clone();
            unconfirmed_blocks.push(unconfirmed_block);
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
        }
    }

    // Find the common ancestor of the latest header and main chain
    pub async fn lookup_common_ancestor_in_eth(
        &mut self,
        blocks: &[EthUnConfirmedBlock],
        mut index: isize,
    ) -> Result<Block<H256>> {
        while index >= 0 {
            let latest_block = &blocks[index as usize];
            let block = self
                .eth_client
                .get_block(U64::from(latest_block.number).into())
                .await;
            if block.is_ok() {
                let block = block.unwrap();
                if hex::encode(
                    block
                        .hash
                        .ok_or_else(|| anyhow!("the block hash is not exist."))?,
                ) == latest_block.hash
                {
                    return Ok(block);
                }
            }
            // The latest header on ckb is not on the Ethereum main chain and needs to be backtracked
            index -= 1;
        }
        anyhow::bail!("system error! can not find the common ancestor with main chain.")
    }

    pub async fn handle_lock_event(
        &mut self,
        records: &mut Vec<EthToCkbRecord>,
        contract_addr: String,
        eth_spv_proof: &EthSpvProof,
        block_number: u64,
    ) -> Result<()> {
        let eth_proof_json = to_eth_spv_proof_json(
            eth_spv_proof.clone(),
            contract_addr,
            String::from(self.eth_client.url()),
        )
        .await?;
        let proof_json = serde_json::to_string(&eth_proof_json)?;
        let recipient_lockscript = hex::encode(eth_proof_json.recipient_lockscript);
        if self.indexer_filter.filter(recipient_lockscript.clone()) {
            let record = EthToCkbRecord {
                eth_lock_tx_hash: String::from(clear_0x(eth_spv_proof.tx_hash.clone().as_str())),
                status: "pending".to_string(),
                token_addr: Some(hex::encode(eth_spv_proof.token.as_bytes())),
                ckb_recipient_lockscript: Some(recipient_lockscript),
                locked_amount: Some(Uint128::from(eth_spv_proof.lock_amount).to_string()),
                eth_spv_proof: Some(proof_json),
                replay_resist_outpoint: Some(hex::encode(
                    eth_spv_proof.replay_resist_outpoint.as_slice(),
                )),
                eth_block_number: block_number,
                sender_addr: Some(hex::encode(eth_spv_proof.sender.as_bytes())),
                sudt_extra_data: Some(hex::encode(eth_spv_proof.sudt_extra_data.as_slice())),
                bridge_fee: Some(Uint128::from(eth_spv_proof.bridge_fee).to_string()),
                ..Default::default()
            };
            records.push(record);
        }
        Ok(())
    }

    pub fn parse_event_with_retry(
        &mut self,
        hash: String,
        max_retry_times: i32,
        contract_addr: String,
    ) -> Result<(Vec<EthSpvProof>, Vec<UnlockEvent>)> {
        let hash_with_0x = format!("{}{}", "0x", hash);
        for retry in 0..max_retry_times {
            let ret = parse_event(
                self.eth_client.url(),
                contract_addr.clone().as_str(),
                hash_with_0x.clone().as_str(),
            );
            match ret {
                Ok(ret) => return Ok(ret),
                Err(e) => {
                    info!("parse event failed, retried {} times, err: {}", retry, e);
                    if e.to_string().contains("the event is not exist") {
                        info!("the event tx is not exist");
                        return Ok(Default::default());
                    }
                }
            }
        }
        Err(anyhow!(
            "Failed to parse event for block hash:{}, after retry {} times",
            hash.as_str(),
            max_retry_times
        ))
    }

    pub fn get_eth_spv_proof_with_retry(
        &mut self,
        hash: String,
        max_retry_times: i32,
        contract_addr: String,
    ) -> Result<(EthSpvProof, bool)> {
        for retry in 0..max_retry_times {
            let ret = generate_eth_proof(
                hash.clone(),
                String::from(self.eth_client.url()).clone(),
                contract_addr.clone(),
            );
            match ret {
                Ok(proof) => return Ok((proof, true)),
                Err(e) => {
                    info!(
                        "get eth receipt proof failed, retried {} times, err: {}",
                        retry, e
                    );
                    if e.to_string().contains("the locked tx is not exist") {
                        info!("the locked tx is not exist");
                        return Ok((Default::default(), false));
                    }
                }
            }
        }
        Err(anyhow!(
            "Failed to generate eth proof for lock tx:{}, after retry {} times",
            hash.as_str(),
            max_retry_times
        ))
    }

    pub async fn handle_unlock_event(
        &mut self,
        tx_hash_str: String,
        unlock_datas: &mut Vec<(String, String)>,
    ) -> Result<()> {
        let tx_hash = convert_hex_to_h256(&tx_hash_str)?;
        let tx = self
            .eth_client
            .client()
            .eth()
            .transaction(tx_hash.into())
            .await?;
        if tx.is_some() {
            let input = tx.unwrap().input;
            let function = Function {
                name: "unlockToken".to_owned(),
                inputs: vec![
                    Param {
                        name: "ckbTxProof".to_owned(),
                        kind: ParamType::Bytes,
                    },
                    Param {
                        name: "ckbTx".to_owned(),
                        kind: ParamType::Bytes,
                    },
                ],
                outputs: vec![],
                constant: false,
            };
            let input_data = function.decode_input(input.0[4..].as_ref())?;
            let ckb_tx = input_data[1].clone();
            let tx_raw = &ckb_tx.to_bytes();
            if tx_raw.is_some() {
                let ckb_tx_hash = blake2b_256(tx_raw.as_ref().unwrap().as_slice());
                let ckb_tx_hash_str = hex::encode(ckb_tx_hash);
                unlock_datas.push((
                    ckb_tx_hash_str,
                    String::from(clear_0x(tx_hash_str.clone().as_str())),
                ));
            }
        }
        Ok(())
    }

    pub fn parse_unlock_event_with_retry(
        &mut self,
        hash: String,
        max_retry_times: i32,
        contract_addr: String,
    ) -> Result<(UnlockEvent, bool)> {
        for retry in 0..max_retry_times {
            let ret = parse_unlock_event(
                hash.clone(),
                String::from(self.eth_client.url()).clone(),
                contract_addr.clone(),
            );
            match ret {
                Ok(event) => return Ok((event, true)),
                Err(e) => {
                    info!(
                        "get eth receipt proof failed, retried {} times, err: {}",
                        retry, e
                    );
                    if e.to_string().contains("the unlocked tx is not exist") {
                        info!("the unlocked tx is not exist");
                        return Ok((Default::default(), false));
                    }
                }
            }
        }
        Err(anyhow!(
            "Failed to parse unlock event for tx:{}, after retry {} times",
            hash.as_str(),
            max_retry_times
        ))
    }
}

#[test]
fn test_decode() {
    let input = "0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000aaaa0000001c0000001e0000002600000046000000660000008600000002003d0000000000000087665cdcc219b392791360a8077fb12e37a43554434f1694026a2ad4ecae078ea4f6038b2f6b634d0aa46cd45be2880cf89153f7866aa7e857f91e4f60da69e584635bda360a131d909a63dd21b4ff2f757edcfbb0e43748520959c37aac910b01000000404bb346c38c5efb4471763d1c7771085bea1c3bd4d9d8509d8f692f8e43e80600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000039d9d0300001c00000020000000b8000000bc00000018010000d50200000000000004000000e287ea8bc97eceaf3420e780b7341c739128da626b1d4158923820727ada5e7a0000000000e287ea8bc97eceaf3420e780b7341c739128da626b1d4158923820727ada5e7a0200000000e287ea8bc97eceaf3420e780b7341c739128da626b1d4158923820727ada5e7a0400000000a777fd1964ffa98a7b0b6c09ff71691705d84d5ed1badfb14271a3a870bdd06b000000000100000000020000000000000000000000fd6edeff40306873d838681e8020aee3ac5080c63a1235de3102767ec6a0d3750000000000000000000000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f810a000000bd01000010000000a60000005c0100009600000010000000180000006100000000ba1dd205000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c35000000100000003000000031000000ed0df97ea89ce848b20479194c9eb50cda612837f2db516b828ffeea61473ff30000000000b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c55000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df4190020000000b5ff94e85f04396cf5b852446eb75d8880cad4d94a1c17d0e5cd70470e6c2ba86100000010000000180000006100000080e4c47b606dc11b490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171cc800000010000000b0000000c40000009c000000403a53a7dfa7a4ab022e53feff11232b3140407d0000000000000000000000000000000000000000cd62e77cfe0386343c15c13528675aae9925d7ae88d9ffc645fef37c2097140cdc2923726d4efe16131e76e85757b446138e39ceda6d3ad483fb11a5619e65035c3139acdb17c26e73647b7f0ac62a4036ca4e720200000000000000000000000000000001000000000000000000000000000000100000005a00000000000000000000000000000000000000000000";
    let input_bin = hex::decode(input).unwrap();
    let function = Function {
        name: "unlockToken".to_owned(),
        inputs: vec![
            Param {
                name: "ckbTxProof".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "ckbTx".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.decode_input(input_bin.as_slice()).unwrap();
    let ckb_tx = input_data[1].clone();
    let tx_raw = &ckb_tx.to_bytes();
    if tx_raw.is_some() {
        let ckb_tx_hash = blake2b_256(tx_raw.as_ref().unwrap().as_slice());
        let hash = hex::encode(ckb_tx_hash);
        assert_eq!(
            hash,
            "a4f6038b2f6b634d0aa46cd45be2880cf89153f7866aa7e857f91e4f60da69e5"
        )
    }
}
