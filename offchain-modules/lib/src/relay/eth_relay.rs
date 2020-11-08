use crate::util::ckb_util::Generator;
use crate::util::eth_proof_helper::{read_block, Witness};
use crate::util::eth_util::Web3Client;
use crate::util::settings::Settings;
use anyhow::{anyhow, Result};
use ckb_sdk::{AddressPayload, SECP256K1};
use ckb_types::bytes::Bytes;
use ckb_types::core::DepType;
use ckb_types::packed::{self, ScriptReader};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Builder, Entity, Reader};
use cmd_lib::run_cmd;
use ethereum_types::{H256, U64};
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::Cell;
use force_sdk::tx_helper::sign;
use force_sdk::util::{parse_privkey_path, send_tx_sync};
use secp256k1::SecretKey;
use std::ops::Add;
use web3::types::{Block, BlockHeader, BlockId};

pub const INIT_ETH_HEIGHT: u64 = 10000;

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
    ) -> Self {
        let settings = Settings::new(&config_path).expect("can not init settings.");
        let generator = Generator::new(ckb_rpc_url, indexer_url, settings)
            .map_err(|e| anyhow::anyhow!(e))
            .unwrap();
        let eth_client = Web3Client::new(eth_rpc_url);
        let temp_typescript = parse_cell(&cell);
        let cell_typescript;
        match temp_typescript {
            Err(_) => cell_typescript = None,
            Ok(temp_typescript) => cell_typescript = Some(temp_typescript),
        }

        ETHRelayer {
            eth_client,
            generator,
            priv_key_path,
            proof_data_path,
            cell_typescript,
        }
    }

    // 1. Query the latest block height of the current main chain of the ckb contract: latest_height
    // 2. current header current_height = latest_height + 1
    // 3. Determine if reorg occurs
    // 4. If reorg occurs, start backtracking and find the common ancestor common_ancestor_height,
    // current_height = common_ancestor_height + 1
    // 5. If reorg does not occur, directly use header as tip to build output
    pub async fn start(&mut self) -> Result<()> {
        match &self.cell_typescript {
            None => self.do_first_relay().await?,
            Some(script) => {
                let typescript = Script::new_builder()
                    .code_hash(script.code_hash())
                    .hash_type(script.hash_type())
                    .args(script.args())
                    .build();
                let cell =
                    get_live_cell_by_typescript(&mut self.generator.indexer_client, typescript)
                        .unwrap()
                        .unwrap();

                let ckb_cell_data = packed::Bytes::from(cell.clone().output_data).raw_data();
                let headers = parse_headers(ckb_cell_data)?;
                let index = headers.len() - 1;
                // Determine whether the latest_header is on the Ethereum main chain
                // If it is in the main chain, the new header currently needs to be added current_height = latest_height + 1
                // If it is not in the main chain, it means that reorg has occurred, and you need to trace back from latest_height until the back traced header is on the main chain
                let block = self.lookup_common_ancestor(&headers, index).await?;
                self.do_relay_loop(block, &cell, &headers).await?;
            }
        }
        Ok(())
    }

    pub async fn do_first_relay(&mut self) -> Result<()> {
        let typescript = Script::new_builder()
            .code_hash(
                Byte32::from_slice(
                    self.generator
                        .settings
                        .light_client_typescript
                        .code_hash
                        .as_bytes(),
                )
                .unwrap(),
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
        let unsigned_tx = self
            .generator
            .init_light_client_tx(&witness, from_lockscript, typescript)
            .unwrap();
        let tx = sign(unsigned_tx, &mut self.generator.rpc_client, &from_privkey).unwrap();
        send_tx_sync(&mut self.generator.rpc_client, &tx, 60).unwrap();
        Ok(())
    }

    pub fn generate_witness(&mut self, header_rlp: String) -> Result<Witness> {
        println!("{:?}", header_rlp);
        let proof_data_path = self.proof_data_path.clone();
        run_cmd!(../vendor/relayer ${header_rlp} > ${proof_data_path})?;
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
        mut index: usize,
    ) -> Result<Block<H256>> {
        while index > 0 {
            let latest_header = &headers[index];
            let block = self
                .eth_client
                .get_block(BlockId::Hash(latest_header.hash.unwrap()))
                .await;
            match block {
                Ok(block) => {
                    return Ok(block);
                }
                Err(_) => {
                    // The latest header on ckb is not on the Ethereum main chain and needs to be backtracked
                    index -= 1;
                }
            }
        }
        Err(anyhow::Error::msg(
            "system error! can not find the common ancestor with main chain.",
        ))
    }

    pub async fn do_relay_loop(
        &mut self,
        mut current_block: Block<H256>,
        cell: &Cell,
        headers: &[BlockHeader],
    ) -> Result<()> {
        let mut number = current_block.number.unwrap();
        loop {
            // let block_id = BlockId::Number(BlockNumber::Number((number.as_u64().add(1)).into()));
            let new_header = self
                .eth_client
                .get_block(number.add(1 as u64).into())
                .await?;
            if new_header.parent_hash == current_block.hash.unwrap() {
                // No reorg
                let new_header_rlp = self.eth_client.get_header_rlp(number.add(1).into()).await?;
                let header_rlp = format!("0x{}", new_header_rlp);
                let witness = self.generate_witness(header_rlp)?;
                let from_privkey = parse_privkey_path(self.priv_key_path.as_str())?;
                let from_lockscript = self.generate_from_lockscript(from_privkey)?;
                let unsigned_tx = self
                    .generator
                    .generate_eth_light_client_tx(
                        &new_header,
                        cell,
                        &witness,
                        headers,
                        from_lockscript,
                    )
                    .unwrap();
                let tx = sign(unsigned_tx, &mut self.generator.rpc_client, &from_privkey).unwrap();
                send_tx_sync(&mut self.generator.rpc_client, &tx, 60).unwrap();
                // FIXME: update cell
                number = number.add(1);
                current_block = new_header;
            } else {
                // Reorg occurred, need to go back
                let index = headers.len() - 1;
                current_block = self.lookup_common_ancestor(&headers, index).await?;
                number = current_block.number.unwrap();
            }

            // send ckb tx
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
}

pub fn parse_headers(_data: Bytes) -> Result<Vec<BlockHeader>> {
    todo!()
}

pub fn parse_cell(cell: &str) -> Result<Script> {
    let cell_bytes =
        hex::decode(cell).map_err(|e| anyhow!("cell shoule be hex format, err: {}", e))?;
    ScriptReader::verify(&cell_bytes, false).map_err(|e| anyhow!("cell decoding err: {}", e))?;
    let cell_typescript = Script::new_unchecked(cell_bytes.into());
    Ok(cell_typescript)
}
