use crate::util::ckb_tx_generator::{Generator, CONFIRM};
use crate::util::ckb_util::{clear_0x, parse_cell, parse_main_chain_headers, parse_privkey_path};
use crate::util::config::ForceConfig;
use crate::util::eth_proof_helper::{read_block, Witness};
use crate::util::eth_util::Web3Client;
use anyhow::{anyhow, Result};
use ckb_sdk::constants::MULTISIG_TYPE_HASH;
use ckb_sdk::{Address, AddressPayload, SECP256K1};
use ckb_types::core::{ScriptHashType, TransactionView};
use ckb_types::packed::Script;
use ckb_types::prelude::{Builder, Entity, Pack};
use cmd_lib::run_cmd;
use ethereum_types::H256;
use force_sdk::cell_collector::get_live_cell_by_lockscript;
use force_sdk::indexer::{Cell, IndexerRpcClient};
use force_sdk::tx_helper::{sign_with_multi_key, MultisigConfig};
use log::{debug, info};
use secp256k1::SecretKey;
use shellexpand::tilde;
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
    pub secret_key: SecretKey,
}

impl ETHRelayer {
    pub fn new(
        config_path: String,
        network: Option<String>,
        priv_key_path: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let mut force_config = ForceConfig::new(config_path.as_str())?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;

        let mut generator =
            Generator::new(ckb_rpc_url, ckb_indexer_url, deployed_contracts.clone())
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
            config: force_config,
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
        let cell = get_live_cell_by_lockscript(&mut self.generator.indexer_client, cell_script)
            .map_err(|err| anyhow::anyhow!(err))?
            .ok_or_else(|| anyhow::anyhow!("no cell found"))?;
        self.do_relay_loop(cell).await?;
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
    let cell_lockscript = tx
        .output(0)
        .ok_or_else(|| anyhow!("no out_put found"))?
        .lock();
    for i in 0..timeout {
        let temp_cell = get_live_cell_by_lockscript(index_client, cell_lockscript.clone())
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
                tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
            }
        }
    }
    i = 0;
    loop {
        let cell_res =
            get_live_cell_by_lockscript(&mut generator.indexer_client, cell_script.clone());
        let cell;
        match cell_res {
            Ok(cell_op) => {
                if cell_op.is_none() {
                    info!("waiting for finding cell deps, loop index: {}", i);
                    tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                    i += 1;
                    continue;
                }
                cell = cell_op.unwrap();
            }
            Err(_) => {
                debug!("waiting for finding cell deps, loop index: {}", i);
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                i += 1;
                continue;
            }
        }

        let ckb_cell_data = cell.clone().output_data.as_bytes().to_vec();
        let (un_confirmed_headers, _) = parse_main_chain_headers(ckb_cell_data)?;

        let best_block_height = un_confirmed_headers[un_confirmed_headers.len() - 1]
            .number
            .unwrap()
            .as_u64();
        if best_block_height > header.number
            && (best_block_height - header.number) as usize >= CONFIRM
        {
            break;
        }

        info!(
            "waiting for eth client header reach sync, eth header number: {:?}, ckb light client number: {:?}, loop index: {}",
            header.number, un_confirmed_headers[un_confirmed_headers.len() - 1].number.unwrap().as_u64(),i,
        );
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        i += 1;
    }

    Ok(())
}
