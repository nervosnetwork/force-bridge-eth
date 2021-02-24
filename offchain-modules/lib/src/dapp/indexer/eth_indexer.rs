use crate::dapp::db::indexer::{
    create_eth_to_ckb_record, delete_eth_to_ckb_records, delete_eth_unconfirmed_block,
    get_eth_unconfirmed_block, get_eth_unconfirmed_blocks, get_height_info,
    get_max_eth_unconfirmed_block, insert_eth_unconfirmed_block, insert_eth_unconfirmed_blocks,
    is_ckb_to_eth_record_exist, reset_ckb_to_eth_record_status, update_ckb_to_eth_record_status,
    update_cross_chain_height_info, update_eth_unconfirmed_block, CrossChainHeightInfo,
    EthToCkbRecord, EthUnConfirmedBlock,
};
use crate::dapp::indexer::IndexerFilter;
use crate::transfer::to_ckb::to_eth_spv_proof_json;
use crate::util::ckb_util::{clear_0x, parse_cell, parse_merkle_cell_data};
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_hex_to_h256, Web3Client};
use crate::util::generated::ckb_tx_proof::CKBUnlockTokenParamReader;
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::Uint128;
use ethabi::{Function, Param, ParamType};
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::IndexerRpcClient;
use log::info;
use molecule::prelude::Reader;
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
        indexer_filter: T,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let indexer_client = IndexerRpcClient::new(ckb_indexer_url);
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
            let (start_height, latest_height, _) = parse_merkle_cell_data(ckb_cell_data.to_vec())?;
            log::info!(
                "get_light_client_height start_height: {:?}, latest_height: {:?}",
                start_height,
                latest_height
            );

            return Ok(latest_height);
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
            log::info!("init cross chain height info.");
            height_info.height = self.get_light_client_height_with_loop().await;
        }
        let mut start_block_number = height_info.height + 1;
        let mut unconfirmed_blocks = get_eth_unconfirmed_blocks(&self.db).await?;
        if unconfirmed_blocks.is_empty() {
            // init unconfirmed_blocks
            log::info!("init eth unconfirmed blocks.");
            self.init_eth_unconfirmed_blocks(&mut unconfirmed_blocks, start_block_number)
                .await?;
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
                        (unconfirmed_blocks.len() - 1) as isize,
                    )
                    .await?;
                log::info!("find the common ancestor block: {:?}", block);
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
            self.handle_eth_event(
                &block,
                lock_contract_address.clone(),
                &mut start_block_number,
                &mut unconfirmed_blocks,
                &mut tail,
                re_org,
            )
            .await?;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn init_eth_unconfirmed_blocks(
        &mut self,
        unconfirmed_blocks: &mut Vec<EthUnConfirmedBlock>,
        start_block_number: u64,
    ) -> Result<()> {
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
        Ok(())
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

    // handle eth event. parse lock event && unlock event.
    pub async fn handle_eth_event(
        &mut self,
        block: &Block<H256>,
        lock_contract_address: String,
        start_block_number: &mut u64,
        unconfirmed_blocks: &mut Vec<EthUnConfirmedBlock>,
        tail: &mut EthUnConfirmedBlock,
        re_org: bool,
    ) -> Result<()> {
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
                    *start_block_number,
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
        // height_info = get_height_info(&self.db, 1 as u8).await?;
        let mut unconfirmed_block;
        let hash_str = hex::encode(
            block
                .hash
                .ok_or_else(|| anyhow!("the block is not exist"))?,
        );
        if unconfirmed_blocks.len() < ETH_CHAIN_CONFIRMED {
            unconfirmed_block = EthUnConfirmedBlock {
                id: *start_block_number % ETH_CHAIN_CONFIRMED as u64,
                number: *start_block_number,
                hash: hash_str,
            };
        } else {
            unconfirmed_block = get_eth_unconfirmed_block(
                &self.db,
                *start_block_number % ETH_CHAIN_CONFIRMED as u64,
            )
            .await?
            .ok_or_else(|| anyhow!("the block is not exist"))?;
            unconfirmed_block.number = *start_block_number;
            unconfirmed_block.hash = hash_str;
        }
        self.write_to_db(
            re_org,
            *start_block_number,
            lock_records,
            unlock_records,
            unconfirmed_blocks,
            &unconfirmed_block,
        )
        .await?;
        *start_block_number += 1;
        *tail = unconfirmed_block.clone();
        unconfirmed_blocks.push(unconfirmed_block);
        Ok(())
    }

    pub async fn write_to_db(
        &mut self,
        re_org: bool,
        start_block_number: u64,
        lock_records: Vec<EthToCkbRecord>,
        unlock_records: Vec<(String, String)>,
        unconfirmed_blocks: &mut Vec<EthUnConfirmedBlock>,
        unconfirmed_block: &EthUnConfirmedBlock,
    ) -> Result<()> {
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
        Ok(())
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
                token_addr: hex::encode(eth_spv_proof.token.as_bytes()),
                ckb_recipient_lockscript: recipient_lockscript,
                locked_amount: Uint128::from(eth_spv_proof.lock_amount).to_string(),
                eth_spv_proof: Some(proof_json),
                replay_resist_outpoint: hex::encode(
                    eth_spv_proof.replay_resist_outpoint.as_slice(),
                ),
                eth_block_number: block_number,
                sender_addr: hex::encode(eth_spv_proof.sender.as_bytes()),
                sudt_extra_data: Some(hex::encode(eth_spv_proof.sudt_extra_data.as_slice())),
                bridge_fee: Uint128::from(eth_spv_proof.bridge_fee).to_string(),
                ..Default::default()
            };
            records.push(record);
        }
        info!("handle lock event. records: {:?}", records);
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
                inputs: vec![Param {
                    name: "proof".to_owned(),
                    kind: ParamType::Bytes,
                }],
                outputs: vec![],
                constant: false,
            };
            info!("input: {:?}", hex::encode(input.0.clone()));
            let input_data = function.decode_input(input.0[4..].as_ref())?;
            let ckb_tx_proof_token = input_data[0].clone();
            let ckb_tx_proof_raw = ckb_tx_proof_token.to_bytes();
            if let Some(ckb_tx_proof_raw) = ckb_tx_proof_raw {
                let raw_data = &ckb_tx_proof_raw;
                CKBUnlockTokenParamReader::verify(raw_data, false).map_err(|err| anyhow!(err))?;
                let ckb_tx_proof_reader = CKBUnlockTokenParamReader::new_unchecked(raw_data);
                let ckb_tx_proof_vec = ckb_tx_proof_reader.tx_proofs();
                let raw_tx = ckb_tx_proof_vec
                    .get_unchecked(0)
                    .raw_transaction()
                    .raw_data();
                let ckb_tx_hash = blake2b_256(raw_tx);
                let ckb_tx_hash_str = hex::encode(ckb_tx_hash);
                if !is_ckb_to_eth_record_exist(&self.db, ckb_tx_hash_str.as_str()).await? {
                    info!("the burn tx is not exist. waiting for ckb indexer reach sync status.");
                    tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                    anyhow::bail!(
                        "the burn tx is not exist. waiting for ckb indexer reach sync status."
                    );
                }
                unlock_datas.push((
                    ckb_tx_hash_str,
                    String::from(clear_0x(tx_hash_str.clone().as_str())),
                ));
            }
        }
        info!("handle lock event. unlock_datas: {:?}", unlock_datas);
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
    let input = "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000577770500000c000000480100003c01000018000000200000002800000034000000580000004b00000000000000a40000000000000001000000a90000000000000001000000782d8a68f24bdc29ea9f8d4ef334391190e229c2079e6150811f81aeb8b55c0607000000c7827748235752a23cbaa0d810d3b837aa11048a8bb482c9d258de4c67ee0f663d1e5decde7ab9e16195a2aba2b071c73cd5580a0032f53b5a5a5d42d3fcfc6ddbb3dad17f27e80feb483584bdc5e20cf184ff95ec8763075d45dd5d4ba5656aa6ae6fa8a846b61b421e50ef37f272fcee11560ae0ef713c2fac84ba67b0752703609c4f2f0ffc7d718c9ba7b9acc4f0363f7f958733b255a9457b2f56b3949632adac6318e47c26f8f11790d7922bd701e8fa01fdc882d1afa2275ee1ab0857403fd148f1e598c92f68a77ba3ab4b686ec8511b44fe78fd1861b598aa0b33642f040000080000002704000018000000200000002200000042000000860000009b000000000000000300e28c25a04a8fd84706a09152b989ce4962c30e9d101857b25e2d0f883c48c032020000001162207f8e96dea1fd5f9b89fc33ba77f9762a8cb40ccad4b94a625f80bb02188c7af64779e83de233ed0f415fe77d83aa104ae9b35ee0850d0bc7be9d6af5909d0300009d0300001c00000020000000b8000000bc00000018010000d502000000000000040000001b9015427d92d2ba3986283c7f6777e63673bd9ed67dc73d4e6f607890646a0200000000001b9015427d92d2ba3986283c7f6777e63673bd9ed67dc73d4e6f607890646a0202000000001b9015427d92d2ba3986283c7f6777e63673bd9ed67dc73d4e6f607890646a020500000000a777fd1964ffa98a7b0b6c09ff71691705d84d5ed1badfb14271a3a870bdd06b0000000001000000000200000000000000000000002b67e7490b251e8c21430e4d0ab43586894baf1674dc6543ec24729bafd3b5e1000000000000000000000000979bb5b6f365dd03908d0995c2c2ed535cbf5d6effddd4a0cab3e6a0bed0b43d00000000bd01000010000000a60000005c0100009600000010000000180000006100000000ba1dd205000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000f2a237e5342a0a826326e109b630609456d83c7635000000100000003000000031000000ad5d462324bfc392652ffe2e9cccdfa9ed9f967559acd3883d533e11ff7e5a590000000000b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000f2a237e5342a0a826326e109b630609456d83c7655000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df41900200000006b3dc6dedae32451fa5024eb5e015176a46a878830e9d65cb79879033611e1ab61000000100000001800000061000000809ebde010000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000f2a237e5342a0a826326e109b630609456d83c76c800000010000000b0000000c40000009c00000017c4b5ce0605f63732bfd175fece7ac6b4620fd2000000000000000000000000000000000000000071bb832290b2b79f50e728af846ea9af0fc15a5364fbfb7225e76ca26155b9ec2cbcb6e25c5d49991343ed1ab1536f1b0d429d72ff63902f8814169b4a3fd8850b9c0bca06c21fe6664cc872c604bede1d633535020000000000000000000000000000000100000000000000000000000000000010000000fe7fb7952f5feb0d000000000000000000000000000000000000000000";
    let input_bin = hex::decode(input).unwrap();
    let function = Function {
        name: "unlockToken".to_owned(),
        inputs: vec![Param {
            name: "proof".to_owned(),
            kind: ParamType::Bytes,
        }],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.decode_input(input_bin.as_slice()).unwrap();
    let ckb_tx_proof_token = input_data[0].clone();
    let ckb_tx_proof_raw = ckb_tx_proof_token.to_bytes();
    if let Some(ckb_tx_proof_raw) = ckb_tx_proof_raw {
        let raw_data = &ckb_tx_proof_raw;
        CKBUnlockTokenParamReader::verify(raw_data, false)
            .map_err(|err| anyhow!(err))
            .unwrap();
        let ckb_tx_proof_reader = CKBUnlockTokenParamReader::new_unchecked(raw_data);
        let ckb_tx_proof_vec = ckb_tx_proof_reader.tx_proofs();
        let raw_tx = ckb_tx_proof_vec
            .get_unchecked(0)
            .raw_transaction()
            .raw_data();
        let ckb_tx_hash = blake2b_256(raw_tx);
        let ckb_tx_hash_str = hex::encode(ckb_tx_hash);
        assert_eq!(
            ckb_tx_hash_str,
            "406aed83854743378b9b5a6809be9b28e8eb54035a1ab622fb00ce8b7b9e8548"
        );
    }
}
