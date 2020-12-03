use crate::util::ckb_tx_generator::{Generator, CONFIRM};
use crate::util::ckb_util::{parse_cell, parse_main_chain_headers, parse_privkey_path};
use crate::util::config::ForceConfig;
use crate::util::eth_proof_helper::{read_block, Witness};
use crate::util::eth_util::Web3Client;
use anyhow::{anyhow, Result};
use ckb_sdk::{AddressPayload, SECP256K1};
use ckb_types::core::{ScriptHashType, TransactionView};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Builder, Entity};
use cmd_lib::run_cmd;
use ethereum_types::H256;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::{Cell, IndexerRpcClient};
use force_sdk::tx_helper::sign;
use force_sdk::util::send_tx_sync;
use log::info;
use secp256k1::SecretKey;
use std::ops::Add;
use web3::types::{Block, BlockHeader};

pub const HEADER_LIMIT_IN_TX: usize = 5;

pub struct ETHRelayer {
    pub eth_client: Web3Client,
    pub generator: Generator,
    pub secret_key: SecretKey,
    pub proof_data_path: String,
    pub cell_typescript: Option<Script>,
    pub config_path: String,
    pub config: ForceConfig,
}

impl ETHRelayer {
    pub fn new(
        config_path: String,
        network: Option<String>,
        priv_key_path: String,
        proof_data_path: String,
    ) -> Result<Self> {
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
        let cell = &deployed_contracts.light_client_cell_script.cell_script;
        let temp_typescript = parse_cell(&cell);
        let cell_typescript;
        match temp_typescript {
            Err(_) => cell_typescript = None,
            Ok(temp_typescript) => cell_typescript = Some(temp_typescript),
        }
        let secret_key = parse_privkey_path(&priv_key_path, &force_config, &network)?;
        Ok(ETHRelayer {
            eth_client,
            generator,
            secret_key,
            proof_data_path,
            cell_typescript,
            config_path,
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
        let typescript;
        // The first relay will generate a unique typescript, and subsequent relays will always use this typescript.
        match &self.cell_typescript {
            None => {
                let cell_script = self.do_first_relay().await?;
                typescript = Script::new_builder()
                    .code_hash(cell_script.code_hash())
                    .hash_type(cell_script.hash_type())
                    .args(cell_script.args())
                    .build();
                self.generator
                    .deployed_contracts
                    .light_client_cell_script
                    .cell_script = hex::encode(typescript.clone().as_slice());
                self.config.deployed_contracts = Some(self.generator.deployed_contracts.clone());
                self.config.write(&self.config_path)?;
                self.cell_typescript = Some(cell_script);
            }
            Some(cell_script) => {
                typescript = Script::new_builder()
                    .code_hash(cell_script.code_hash())
                    .hash_type(cell_script.hash_type())
                    .args(cell_script.args())
                    .build();
            }
        }
        println!(
            "start cell typescript: \n{}",
            serde_json::to_string_pretty(&ckb_jsonrpc_types::Script::from(typescript.clone()))
                .map_err(|err| anyhow!(err))?
        );
        tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
        // get the latest output cell
        let cell = get_live_cell_by_typescript(&mut self.generator.indexer_client, typescript)
            .map_err(|err| anyhow::anyhow!(err))?
            .ok_or_else(|| anyhow::anyhow!("no cell found"))?;

        self.do_relay_loop(cell).await?;
        Ok(())
    }

    //The first time the relay uses the outpoint of the first input when it is created,
    // to ensure that the typescript is unique across the network
    pub async fn do_first_relay(&mut self) -> Result<Script> {
        let typescript = Script::new_builder()
            .code_hash(
                Byte32::from_slice(
                    hex::decode(
                        &self
                            .generator
                            .deployed_contracts
                            .light_client_typescript
                            .code_hash,
                    )?
                    .as_slice(),
                )
                .map_err(|err| anyhow::anyhow!(err))?,
            )
            .hash_type(ScriptHashType::Data.into())
            .build();

        let lockscript = Script::new_builder()
            .code_hash(
                Byte32::from_slice(
                    hex::decode(
                        &self
                            .generator
                            .deployed_contracts
                            .light_client_lockscript
                            .code_hash,
                    )?
                    .as_slice(),
                )
                .map_err(|err| anyhow::anyhow!(err))?,
            )
            .hash_type(ScriptHashType::Data.into())
            .build();
        let current_number = self.eth_client.client().eth().block_number().await?;
        let block = self.eth_client.get_block(current_number.into()).await?;
        // let witness = self.generate_witness(block.number.unwrap().as_u64())?;
        let witness = Witness {
            cell_dep_index_list: vec![],
            header: vec![],
            merkle_proof: vec![],
        };
        let from_privkey = self.secret_key;
        let from_lockscript = self.generate_from_lockscript(from_privkey)?;
        let unsigned_tx = self.generator.init_light_client_tx(
            &block,
            &witness,
            from_lockscript,
            typescript,
            lockscript,
        )?;
        let tx = sign(unsigned_tx, &mut self.generator.rpc_client, &from_privkey)
            .map_err(|err| anyhow::anyhow!(err))?;
        send_tx_sync(&mut self.generator.rpc_client, &tx, 60)
            .await
            .map_err(|err| anyhow::anyhow!(err))?;

        let cell_typescript = tx
            .output(0)
            .ok_or_else(|| anyhow!("no out_put found"))?
            .type_()
            .to_opt()
            .ok_or_else(|| anyhow!("cell_typescript is not found."))?;
        println!(
            "first relay cell typescript: \n{}",
            serde_json::to_string_pretty(&ckb_jsonrpc_types::Script::from(cell_typescript.clone()))
                .map_err(|err| anyhow!(err))?
        );
        Ok(cell_typescript)
    }

    pub fn generate_witness(&mut self, number: u64) -> Result<Witness> {
        let proof_data_path = self.proof_data_path.clone();
        run_cmd!(vendor/relayer ${number} > data/proof_data_temp.json)?;
        run_cmd!(tail -1 data/proof_data_temp.json > ${proof_data_path})?;
        let block_with_proofs = read_block(proof_data_path);
        let witness = Witness {
            cell_dep_index_list: vec![0],
            header: block_with_proofs.header_rlp.0.clone(),
            merkle_proof: block_with_proofs.to_double_node_with_merkle_proof_vec(),
        };
        Ok(witness)
    }

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
                        .hash
                        .ok_or_else(|| anyhow!("the block hash is not exist."))?
                        .into(),
                )
                .await;
            if block.is_err() {
                // The latest header on ckb is not on the Ethereum main chain and needs to be backtracked
                index -= 1;
                continue;
            }
            return Ok(block.unwrap());
        }
        anyhow::bail!("system error! can not find the common ancestor with main chain.")
    }

    pub async fn do_relay_loop(&mut self, mut cell: Cell) -> Result<()> {
        let ckb_cell_data = cell.clone().output_data.as_bytes().to_vec();
        let (mut un_confirmed_headers, _) = parse_main_chain_headers(ckb_cell_data)?;
        let index: isize = (un_confirmed_headers.len() - 1) as isize;
        // Determine whether the latest_header is on the Ethereum main chain
        // If it is in the main chain, the new header currently needs to be added current_height = latest_height + 1
        // If it is not in the main chain, it means that reorg has occurred, and you need to trace back from latest_height until the back traced header is on the main chain
        let mut current_block = self
            .lookup_common_ancestor(&un_confirmed_headers, index)
            .await?;
        let mut number = current_block
            .number
            .ok_or_else(|| anyhow!("the block number is not exist."))?;
        loop {
            let witnesses = vec![];
            let start = number.add(1 as u64);
            let end = start.add(HEADER_LIMIT_IN_TX as u64);
            let headers_result = self
                .eth_client
                .get_blocks(start.as_u64(), end.as_u64())
                .await;
            if headers_result.is_err() {
                info!("current block is newest, waiting for new header on ethereum.");
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                continue;
            }
            let headers = headers_result.unwrap();
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

            let from_lockscript = self.generate_from_lockscript(self.secret_key)?;
            let unsigned_tx = self.generator.generate_eth_light_client_tx(
                &headers,
                &cell,
                &witnesses,
                &un_confirmed_headers,
                from_lockscript,
            )?;
            let tx = sign(
                unsigned_tx,
                &mut self.generator.rpc_client,
                &self.secret_key,
            )
            .map_err(|err| anyhow::anyhow!(err))?;
            self.generator
                .rpc_client
                .send_transaction(tx.data())
                .map_err(|err| anyhow!(err))?;

            // update cell current_block and number.
            update_cell_sync(&mut self.generator.indexer_client, &tx, 60, &mut cell)
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
    let cell_typescript = tx
        .output(0)
        .ok_or_else(|| anyhow!("no out_put found"))?
        .type_()
        .to_opt()
        .ok_or_else(|| anyhow!("cell_typescript is not found."))?;
    for i in 0..timeout {
        let temp_cell = get_live_cell_by_typescript(index_client, cell_typescript.clone());
        match temp_cell {
            Ok(temp_cell) => {
                if temp_cell.clone().unwrap().block_number.value() > cell.block_number.value() {
                    *cell = temp_cell.unwrap();
                    break;
                }
            }
            _ => {
                info!("waiting for cell to be committed, loop index: {}", i,);
            }
        }
        info!("waiting for cell to be committed, loop index: {}", i,);
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
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
            get_live_cell_by_typescript(&mut generator.indexer_client, cell_script.clone());
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
                info!("waiting for finding cell deps, loop index: {}", i);
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
