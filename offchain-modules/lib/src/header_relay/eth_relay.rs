use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{
    parse_cell, parse_main_chain_headers, parse_merkle_cell_data, parse_privkey_path,
};
use crate::util::config::ForceConfig;
use crate::util::eth_util::Web3Client;
use crate::util::rocksdb;
use anyhow::{anyhow, Result};
use ckb_sdk::{Address, AddressPayload, SECP256K1};
use ckb_types::core::TransactionView;
use ckb_types::packed::Script;
use ethereum_types::H256;
use force_eth_types::generated::eth_header_cell::ETHHeaderCellMerkleDataReader;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::{Cell, IndexerRpcClient};
use force_sdk::tx_helper::{sign_with_multi_key, MultisigConfig};
use force_sdk::util::send_tx_sync_with_response;
use log::{debug, info};
use molecule::prelude::Reader;
use secp256k1::SecretKey;
// use serde::export::Clone;
use shellexpand::tilde;
use sparse_merkle_tree::traits::Value;
use std::ops::Add;
use std::str::FromStr;
use web3::types::{Block, BlockHeader, U64};

pub const HEADER_LIMIT_IN_TX: usize = 14;

pub struct ETHRelayer {
    pub eth_client: Web3Client,
    pub generator: Generator,
    pub config_path: String,
    pub config: ForceConfig,
    pub multisig_config: MultisigConfig,
    pub multisig_privkeys: Vec<SecretKey>,
    pub secret_key: SecretKey,
    pub confirm: u64,
}

impl ETHRelayer {
    pub fn new(
        config_path: String,
        network: Option<String>,
        priv_key_path: String,
        multisig_privkeys: Vec<String>,
        confirm: u64,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;

        let generator = Generator::new(ckb_rpc_url, ckb_indexer_url, deployed_contracts.clone())
            .map_err(|e| anyhow::anyhow!(e))?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let mut addresses = vec![];
        for item in deployed_contracts.multisig_address.addresses.clone() {
            let address = Address::from_str(&item).unwrap();
            addresses.push(address);
        }
        let sighash_addresses = addresses
            .into_iter()
            .map(|address| address.payload().clone())
            .collect::<Vec<_>>();

        let multisig_config = MultisigConfig::new_with(
            sighash_addresses,
            deployed_contracts.multisig_address.require_first_n,
            deployed_contracts.multisig_address.threshold,
        )
        .map_err(|err| anyhow!(err))?;

        let secret_key = parse_privkey_path(&priv_key_path, &force_config, &network)?;

        Ok(ETHRelayer {
            eth_client,
            generator,
            config_path,
            multisig_config,
            secret_key,
            multisig_privkeys: multisig_privkeys
                .into_iter()
                .map(|k| parse_privkey_path(&k, &force_config, &network))
                .collect::<Result<Vec<SecretKey>>>()?,
            config: force_config,
            confirm,
        })
    }

    // 1. Query the latest block height of the current main chain of the ckb contract: latest_height
    // 2. current header current_height = latest_height + 1
    // 3. Determine if reorg occurs
    // 4. If reorg occurs, start backtracking and find the common ancestor common_ancestor_height,
    // current_height = common_ancestor_height + 1
    // 5. If reorg does not occur, directly use header as tip to build output
    pub async fn start(&mut self) -> Result<()> {
        // get the latest output cell
        let deployed_contracts = self
            .config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no cell found"))?;
        let cell_script = parse_cell(
            deployed_contracts
                .light_client_cell_script
                .cell_script
                .as_str(),
        )?;
        self.do_naive_relay_loop(cell_script).await?;
        // let cell = get_live_cell_by_lockscript(&mut self.generator.indexer_client, cell_script)
        //     .map_err(|err| anyhow::anyhow!(err))?
        //     .ok_or_else(|| anyhow::anyhow!("no cell found"))?;
        // self.do_relay_loop(cell).await?;
        Ok(())
    }

    // pub fn generate_witness(&mut self, number: u64) -> Result<Witness> {
    //     let proof_data_path = self.proof_data_path.clone();
    //     run_cmd!(vendor/relayer ${number} > data/proof_data_temp.json)?;
    //     run_cmd!(tail -1 data/proof_data_temp.json > ${proof_data_path})?;
    //     let block_with_proofs = read_block(proof_data_path);
    //     let witness = Witness {
    //         cell_dep_index_list: vec![0],
    //         header: block_with_proofs.header_rlp.0.clone(),
    //         merkle_proof: block_with_proofs.to_double_node_with_merkle_proof_vec(),
    //     };
    //     Ok(witness)
    // }

    pub fn generate_from_lockscript(&mut self, from_privkey: SecretKey) -> Result<Script> {
        let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
        let address_payload = AddressPayload::from_pubkey(&from_public_key);
        Ok(Script::from(&address_payload))
    }

    // Find the common ancestor of the latest header and main chain
    pub async fn lookup_common_ancestor(
        &mut self,
        headers: &[BlockHeader],
        mut index: isize,
    ) -> Result<Block<H256>> {
        while index >= 0 {
            let latest_header = &headers[index as usize];
            let block = self
                .eth_client
                .get_block(
                    latest_header
                        .number
                        .ok_or_else(|| anyhow!("this number of block is not exist."))?
                        .into(),
                )
                .await;
            if block.is_ok() {
                let block = block.unwrap();
                if block.hash.unwrap() == latest_header.hash.unwrap() {
                    return Ok(block);
                }
            }
            // The latest header on ckb is not on the Ethereum main chain and needs to be backtracked
            index -= 1;
        }
        anyhow::bail!("system error! can not find the common ancestor with main chain.")
    }

    pub async fn naive_relay(
        &mut self,
        from_lockscript: Script,
        cell_script: Script,
        mut latest_submit_header_number: u64,
    ) -> Result<u64> {
        let tip_header_number: u64 = self
            .eth_client
            .client()
            .eth()
            .block_number()
            .await?
            .as_u64();
        if tip_header_number <= self.confirm {
            info!("waiting for tip_header_number reach confirm limit. tip_header_number: {}, confirm: {}", tip_header_number, self.confirm);
            return Ok(latest_submit_header_number);
        }
        if latest_submit_header_number >= tip_header_number {
            info!("waiting for new eth header. tip_header_number: {}, latest_submit_header_number: {}", tip_header_number, latest_submit_header_number);
            return Ok(latest_submit_header_number);
        }

        let force_config = ForceConfig::new(self.config_path.as_str())?;
        let db_path = force_config.eth_rocksdb_path;
        // make tx
        let cell =
            get_live_cell_by_typescript(&mut self.generator.indexer_client, cell_script.clone())
                .map_err(|err| anyhow::anyhow!(err))?
                .ok_or_else(|| anyhow::anyhow!("no cell found"))?;

        let last_cell_output_data = cell.output_data.as_bytes();

        let mut last_cell_latest_height = 0u64;

        let (start_height, mut smt_tree) = match last_cell_output_data.len() {
            0 => {
                let rocksdb_store = rocksdb::RocksDBStore::new(db_path.clone());
                (
                    tip_header_number - self.confirm,
                    rocksdb::SMT::new(sparse_merkle_tree::H256::zero(), rocksdb_store),
                )
            }
            _ => {
                let (start_height, latest_height, merkle_root) =
                    parse_merkle_cell_data(last_cell_output_data.to_vec())?;
                last_cell_latest_height = latest_height;
                let rocksdb_store = rocksdb::RocksDBStore::open(db_path.clone());
                (
                    start_height,
                    rocksdb::SMT::new(merkle_root.into(), rocksdb_store),
                )
            }
        };

        let confirmed_header_number = tip_header_number - self.confirm;
        let mut index = confirmed_header_number;
        while index >= start_height {
            let block_number = U64([index]);

            let mut key = [0u8; 32];
            let mut height = [0u8; 8];
            height.copy_from_slice(index.to_le_bytes().as_ref());
            key[..8].clone_from_slice(&height);

            let chain_block = self.eth_client.get_block(block_number.into()).await?;
            let chain_block_hash = chain_block
                .hash
                .ok_or_else(|| anyhow!("the block number is not exist."))?;

            let db_block_hash = smt_tree
                .get(&key.into())
                .map_err(|err| anyhow::anyhow!(err))?;

            if db_block_hash.to_h256().as_slice() != chain_block_hash.0.as_ref() {
                smt_tree
                    .update(key.into(), chain_block_hash.0.into())
                    .map_err(|err| anyhow::anyhow!(err))?;
                log::info!("sync eth block {} to cache", index);
            } else {
                break;
            }
            index -= 1;
        }
        log::info!(
            "start relaying headers from {} to {}",
            index + 1,
            confirmed_header_number
        );

        let new_merkle_root = smt_tree.root().as_slice();
        let new_latest_height = confirmed_header_number;
        let unsigned_tx = self.generator.generate_eth_light_client_tx_naive(
            from_lockscript.clone(),
            cell.clone(),
            &new_merkle_root,
            start_height,
            new_latest_height,
        )?;

        let mut privkeys = vec![&self.secret_key];
        privkeys.extend(self.multisig_privkeys.iter());
        let tx = sign_with_multi_key(
            unsigned_tx,
            &mut self.generator.rpc_client,
            privkeys,
            self.multisig_config.clone(),
        )
        .map_err(|err| anyhow::anyhow!(err))?;
        let send_tx_res =
            send_tx_sync_with_response(&mut self.generator.rpc_client, &tx, 180).await;
        if let Err(e) = send_tx_res {
            log::error!(
                "relay eth header from {} to {} failed! err: {}",
                last_cell_latest_height,
                confirmed_header_number,
                e
            );
        } else {
            let rocksdb_store = smt_tree.store_mut();
            rocksdb_store.commit();
            info!(
                "Successfully relayed the headers from {} to {}, tip header {}",
                index, confirmed_header_number, tip_header_number
            );
            latest_submit_header_number = confirmed_header_number;
        }
        Ok(latest_submit_header_number)
    }

    // naive relay method. ignore the context, get the latest 500 headers on chain and replace the
    // light client cell
    pub async fn do_naive_relay_loop(&mut self, cell_script: Script) -> Result<()> {
        let from_privkey = self.secret_key;
        let from_lockscript = self.generate_from_lockscript(from_privkey)?;
        let mut latest_submit_header_number = 0;
        loop {
            let res = self
                .naive_relay(
                    from_lockscript.clone(),
                    cell_script.clone(),
                    latest_submit_header_number,
                )
                .await;
            match res {
                Ok(new_submit_header_number) => {
                    latest_submit_header_number = new_submit_header_number
                }
                Err(e) => log::error!(
                    "unexpected error relay header from {}, err: {}",
                    latest_submit_header_number,
                    e
                ),
            }
            tokio::time::delay_for(std::time::Duration::from_secs(300)).await;
        }
    }

    pub async fn do_relay_loop(&mut self, mut cell: Cell) -> Result<()> {
        let ckb_cell_data = cell.clone().output_data.as_bytes().to_vec();
        let mut un_confirmed_headers = vec![];
        let mut index: isize = 0;
        if !ckb_cell_data.is_empty() {
            let (headers, _) = parse_main_chain_headers(ckb_cell_data)?;
            un_confirmed_headers = headers;
            index = (un_confirmed_headers.len() - 1) as isize;
        }
        let mut number: U64;
        let mut current_block: Block<H256>;
        if index == 0 {
            // first relay
            number = self.eth_client.client().eth().block_number().await?;
            current_block = self.eth_client.get_block(number.into()).await?;
        } else {
            // Determine whether the latest_header is on the Ethereum main chain
            // If it is in the main chain, the new header currently needs to be added current_height = latest_height + 1
            // If it is not in the main chain, it means that reorg has occurred, and you need to trace back from latest_height until the back traced header is on the main chain
            current_block = self
                .lookup_common_ancestor(&un_confirmed_headers, index)
                .await?;
            number = current_block
                .number
                .ok_or_else(|| anyhow!("the block number is not exist."))?;
        }

        loop {
            let witnesses = vec![];
            let start = number.add(1 as u64);
            let mut latest_number = self.eth_client.client().eth().block_number().await?;
            if latest_number <= start {
                info!("current block is newest, waiting for new header on ethereum.");
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                continue;
            }
            if latest_number.as_u64() - start.as_u64() > HEADER_LIMIT_IN_TX as u64 {
                latest_number = start.add(HEADER_LIMIT_IN_TX as u64);
            }
            info!(
                "try to relay eth light client, block height start: {:?}, end: {:?}",
                start.as_u64(),
                latest_number.as_u64()
            );
            let headers = self
                .eth_client
                .get_blocks(start.as_u64(), latest_number.as_u64())
                .await?;
            if headers[0].parent_hash
                == current_block
                    .hash
                    .ok_or_else(|| anyhow!("the block hash is not exist."))?
            {
                // No reorg
                // don't remove it, it will be used in later.

                // for item in headers.clone() {
                //     let witness = self.generate_witness(item.number.unwrap().as_u64())?;
                //     witnesses.push(witness);
                // }
            } else {
                // Reorg occurred, need to go back
                info!("reorg occurred, ready to go back");
                let index: isize = (un_confirmed_headers.len() - 1) as isize;
                current_block = self
                    .lookup_common_ancestor(&un_confirmed_headers, index)
                    .await?;
                info!(
                    "reorg occurred, found the common ancestor. {:?}",
                    current_block
                );
                number = current_block
                    .number
                    .ok_or_else(|| anyhow!("the block number is not exist."))?;
                continue;
            }

            let from_privkey = self.secret_key;
            let from_lockscript = self.generate_from_lockscript(from_privkey)?;

            let unsigned_tx = self.generator.generate_eth_light_client_tx(
                &headers,
                &cell,
                &witnesses,
                &un_confirmed_headers,
                from_lockscript,
            )?;
            // FIXME: waiting for sign server.
            let secret_key_a = parse_privkey_path("0", &self.config, &Option::None)?;
            let secret_key_b = parse_privkey_path("1", &self.config, &Option::None)?;
            let tx = sign_with_multi_key(
                unsigned_tx,
                &mut self.generator.rpc_client,
                vec![&self.secret_key, &secret_key_a, &secret_key_b],
                self.multisig_config.clone(),
            )
            .map_err(|err| anyhow::anyhow!(err))?;
            self.generator
                .rpc_client
                .send_transaction(tx.data())
                .map_err(|err| anyhow!(err))?;

            // update cell current_block and number.
            update_cell_sync(&mut self.generator.indexer_client, &tx, 600, &mut cell)
                .await
                .map_err(|err| anyhow::anyhow!(err))?;
            current_block = headers[headers.len() - 1].clone();
            number = current_block.number.unwrap();
            let ckb_cell_data = cell.clone().output_data.as_bytes().to_vec();
            let (un_confirmed, _) = parse_main_chain_headers(ckb_cell_data)?;
            un_confirmed_headers = un_confirmed;
            info!(
                "Successfully relayed the headers, ready to relay the next one. next number: {:?}",
                number
            );
        }
    }
}

pub async fn update_cell_sync(
    index_client: &mut IndexerRpcClient,
    tx: &TransactionView,
    timeout: u64,
    cell: &mut Cell,
) -> Result<()> {
    let cell_script = tx
        .output(0)
        .ok_or_else(|| anyhow!("no out_put found"))?
        .type_()
        .to_opt()
        .ok_or_else(|| anyhow!("no typescript found"))?;
    for i in 0..timeout {
        let temp_cell = get_live_cell_by_typescript(index_client, cell_script.clone())
            .map_err(|e| anyhow!("failed to get temp_cell: {}", e))?;
        if let Some(c) = temp_cell {
            if c.block_number.value() > cell.block_number.value() {
                *cell = c;
                return Ok(());
            }
        }
        info!("waiting for cell to be committed, loop index: {}", i);
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
    anyhow::bail!(
        "failed to update cell after waiting for {} secends. please try again.",
        timeout
    )
}

pub async fn wait_header_sync_success(
    generator: &mut Generator,
    light_client_cell_script: &str,
    header_rlp: String,
) -> Result<()> {
    let header: eth_spv_lib::eth_types::BlockHeader = rlp::decode(
        hex::decode(header_rlp.as_str())
            .unwrap()
            .to_vec()
            .as_slice(),
    )
    .unwrap();
    let mut i = 0;
    let cell_script;
    loop {
        let cell_script_result = parse_cell(light_client_cell_script);
        match cell_script_result {
            Ok(cell_script_result) => {
                cell_script = cell_script_result;
                break;
            }
            Err(_) => {
                info!("waiting for cell script init, loop index: {}", i);
                i += 1;
                tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
            }
        }
    }
    i = 0;
    loop {
        let cell_res =
            get_live_cell_by_typescript(&mut generator.indexer_client, cell_script.clone());
        let cell;
        match cell_res {
            Ok(cell_op) => {
                if cell_op.is_none() {
                    info!("waiting for finding cell deps, loop index: {}", i);
                    tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
                    i += 1;
                    continue;
                }
                cell = cell_op.unwrap();
            }
            Err(_) => {
                debug!("waiting for finding cell deps, loop index: {}", i);
                tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
                i += 1;
                continue;
            }
        }
        let ckb_cell_data = cell.clone().output_data.as_bytes().to_vec();
        if ckb_cell_data.is_empty() {
            debug!("waiting for eth light client cell init, loop index: {}", i);
            tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
            i += 1;
            continue;
        }

        let cell_data_reader = ETHHeaderCellMerkleDataReader::new_unchecked(&ckb_cell_data);
        let mut best_block_height = [0u8; 8];
        let latest_height_raw = cell_data_reader.latest_height().raw_data();
        best_block_height.copy_from_slice(latest_height_raw);
        let best_block_height = u64::from_le_bytes(best_block_height);

        if best_block_height >= header.number {
            break;
        }

        info!(
            "waiting for eth client header reach sync, eth header number: {:?}, ckb light client number: {:?}, loop index: {}",
            header.number, best_block_height, i,
        );
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        i += 1;
    }

    Ok(())
}
