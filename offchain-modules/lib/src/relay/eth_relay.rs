use crate::util::ckb_util::{parse_cell, Generator};
use crate::util::eth_proof_helper::{read_block, Witness};
use crate::util::eth_util::{decode_block_header, Web3Client};
use crate::util::settings::Settings;
use anyhow::{anyhow, Result};
use ckb_sdk::{AddressPayload, SECP256K1};
use ckb_types::bytes::Bytes;
use ckb_types::core::DepType;
use ckb_types::packed::{self};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Builder, Entity, Reader};
use cmd_lib::run_cmd;
use ethereum_types::{H256, U64};
use force_eth_types::generated::eth_header_cell::ETHChainReader;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::Cell;
use force_sdk::tx_helper::sign;
use force_sdk::util::{parse_privkey_path, send_tx_sync};
use log::info;
use rlp::Rlp;
use secp256k1::SecretKey;
use std::ops::Add;
use web3::types::{Block, BlockHeader};

pub const INIT_ETH_HEIGHT: u64 = 10000;
pub const CONFIRM: usize = 10;

pub struct ETHRelayer {
    pub eth_client: Web3Client,
    pub generator: Generator,
    pub priv_key_path: String,
    pub proof_data_path: String,
    pub cell_typescript: Option<Script>,
}

impl ETHRelayer {
    pub fn new(
        config_path: String,
        ckb_rpc_url: String,
        eth_rpc_url: String,
        indexer_url: String,
        priv_key_path: String,
        proof_data_path: String,
        cell: String,
    ) -> Result<Self> {
        let settings = Settings::new(&config_path)?;
        let generator =
            Generator::new(ckb_rpc_url, indexer_url, settings).map_err(|e| anyhow::anyhow!(e))?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let temp_typescript = parse_cell(&cell);
        let cell_typescript;
        match temp_typescript {
            Err(_) => cell_typescript = None,
            Ok(temp_typescript) => cell_typescript = Some(temp_typescript),
        }

        Ok(ETHRelayer {
            eth_client,
            generator,
            priv_key_path,
            proof_data_path,
            cell_typescript,
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
            }
            Some(cell_script) => {
                typescript = Script::new_builder()
                    .code_hash(cell_script.code_hash())
                    .hash_type(cell_script.hash_type())
                    .args(cell_script.args())
                    .build();
            }
        }
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
                    self.generator
                        .settings
                        .light_client_typescript
                        .code_hash
                        .as_bytes(),
                )
                .map_err(|err| anyhow::anyhow!(err))?,
            )
            .hash_type(DepType::Code.into())
            .build();
        let block = self
            .eth_client
            .get_block(U64::from(INIT_ETH_HEIGHT).into())
            .await?;
        let new_header_rlp = self
            .eth_client
            .get_header_rlp(block.number.unwrap().into())
            .await?;
        let header_rlp = format!("0x{}", new_header_rlp);
        let witness = self.generate_witness(header_rlp)?;
        let from_privkey = parse_privkey_path(self.priv_key_path.as_str())?;
        let from_lockscript = self.generate_from_lockscript(from_privkey)?;
        let unsigned_tx =
            self.generator
                .init_light_client_tx(&witness, from_lockscript, typescript)?;
        let tx = sign(unsigned_tx, &mut self.generator.rpc_client, &from_privkey)
            .map_err(|err| anyhow::anyhow!(err))?;
        send_tx_sync(&mut self.generator.rpc_client, &tx, 60)
            .map_err(|err| anyhow::anyhow!(err))?;
        let cell_typescript = tx
            .output(0)
            .ok_or_else(|| anyhow!("no out_put found"))?
            .type_()
            .to_opt()
            .ok_or_else(|| anyhow!("cell_typescript is not found."))?;
        Ok(cell_typescript)
    }

    pub fn generate_witness(&mut self, header_rlp: String) -> Result<Witness> {
        let proof_data_path = self.proof_data_path.clone();
        run_cmd!(../vendor/relayer ${header_rlp} > ${proof_data_path})?;
        let block_with_proofs = read_block(proof_data_path);
        let witness = Witness {
            cell_dep_index_list: vec![0],
            header: block_with_proofs.header_rlp.0.clone(),
            merkle_proof: block_with_proofs.to_double_node_with_merkle_proof_vec(),
        };
        info!(
            "generate witness for header_rlp. header_rlp: {:?}, witness: {:?}",
            header_rlp, witness
        );
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
        mut index: usize,
    ) -> Result<Block<H256>> {
        while index > 0 {
            let latest_header = &headers[index];
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
        let ckb_cell_data = packed::Bytes::from(cell.clone().output_data).raw_data();
        let (un_confirmed_headers, _) = parse_main_chain_headers(ckb_cell_data)?;
        let index = un_confirmed_headers.len() - 1;
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
            let new_number = number.add(1 as u64);
            let new_header_temp = self.eth_client.get_block(new_number.into()).await;
            if new_header_temp.is_err() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            let new_header = new_header_temp.unwrap();
            if new_header.parent_hash
                == current_block
                    .hash
                    .ok_or_else(|| anyhow!("the block hash is not exist."))?
            {
                // No reorg
                info!(
                    "no reorg occurred, ready to relay new header: {:?}",
                    new_header
                );
                let new_header_rlp = self.eth_client.get_header_rlp(new_number.into()).await?;
                let header_rlp = format!("0x{}", new_header_rlp);
                let witness = self.generate_witness(header_rlp)?;
                let from_privkey = parse_privkey_path(self.priv_key_path.as_str())?;
                let from_lockscript = self.generate_from_lockscript(from_privkey)?;
                let unsigned_tx = self.generator.generate_eth_light_client_tx(
                    &new_header,
                    &cell,
                    &witness,
                    &un_confirmed_headers,
                    from_lockscript,
                )?;
                let tx = sign(unsigned_tx, &mut self.generator.rpc_client, &from_privkey)
                    .map_err(|err| anyhow::anyhow!(err))?;
                send_tx_sync(&mut self.generator.rpc_client, &tx, 60)
                    .map_err(|err| anyhow::anyhow!(err))?;

                // update cell current_block and number.
                let cell_typescript = tx
                    .output(0)
                    .ok_or_else(|| anyhow!("no out_put found"))?
                    .type_()
                    .to_opt()
                    .ok_or_else(|| anyhow!("cell_typescript is not found."))?;
                cell = get_live_cell_by_typescript(
                    &mut self.generator.indexer_client,
                    cell_typescript,
                )
                .map_err(|err| anyhow::anyhow!(err))?
                .ok_or_else(|| anyhow::anyhow!("no cell found"))?;
                number = new_number;
                current_block = new_header;
                info!("Successfully relayed the current header, ready to relay the next one. current_number: {:?}", number);
            } else {
                // Reorg occurred, need to go back
                info!("reorg occurred, ready to go back");
                let index = un_confirmed_headers.len() - 1;
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
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

pub fn parse_main_chain_headers(data: Bytes) -> Result<(Vec<BlockHeader>, Vec<Vec<u8>>)> {
    ETHChainReader::verify(&data, false).map_err(|err| anyhow!(err))?;
    let chain_reader = ETHChainReader::new_unchecked(&data);
    let main_reader = chain_reader.main();
    let mut un_confirmed = vec![];
    let mut confirmed = vec![];
    let len = main_reader.len();
    for i in (0..len).rev() {
        if (len - i) < CONFIRM {
            let rlp = Rlp::new(main_reader.get_unchecked(i).raw_data());
            let header: BlockHeader = decode_block_header(&rlp).map_err(|err| anyhow!(err))?;
            un_confirmed.push(header);
        } else {
            confirmed.push(main_reader.get_unchecked(i).raw_data().to_vec())
        }
    }
    Ok((un_confirmed, confirmed))
}
