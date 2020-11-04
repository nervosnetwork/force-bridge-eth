use crate::util::ckb_util::Generator;
use crate::util::eth_util::Web3Client;
use crate::util::settings::Settings;
use anyhow::Result;
use ckb_types::core::DepType;
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Builder, Entity};
use ethereum_types::H256;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use futures::future::BoxFuture;
use futures::FutureExt;
use web3::types::{Block, BlockHeader, BlockId};

pub struct ETHRelayer {
    pub config_path: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub indexer_url: String,
    pub priv_key_path: String,
}

impl ETHRelayer {
    pub fn new(
        config_path: String,
        ckb_rpc_url: String,
        eth_rpc_url: String,
        indexer_url: String,
        priv_key_path: String,
    ) -> Self {
        ETHRelayer {
            config_path,
            ckb_rpc_url,
            eth_rpc_url,
            indexer_url,
            priv_key_path,
        }
    }

    // 1. Query the latest block height of the current main chain of the ckb contract: latest_height
    // 2. current header current_height = latest_height + 1
    // 3. Determine if reorg occurs
    // 4. If reorg occurs, start backtracking and find the common ancestor common_ancestor_height,
    // current_height = common_ancestor_height + 1
    // 5. If reorg does not occur, directly use header as tip to build output
    pub async fn start(&mut self) -> Result<()> {
        let settings = Settings::new(&self.config_path)?;
        let mut generator =
            Generator::new(self.ckb_rpc_url.clone(), self.indexer_url.clone(), settings)
                .map_err(|e| anyhow::anyhow!(e))?;
        let typescript = Script::new_builder()
            .code_hash(
                Byte32::from_slice(generator.settings.typescript.code_hash.as_bytes()).unwrap(),
            )
            .hash_type(DepType::Code.into())
            // FIXME: add script args
            .args(ckb_types::packed::Bytes::default())
            .build();
        let indexer_client = &mut generator.indexer_client;
        let cell = get_live_cell_by_typescript(indexer_client, typescript).unwrap();
        let mut rpc_client = Web3Client::new(self.eth_rpc_url.clone());
        match cell {
            Some(cell) => {
                let data = cell.output_data.as_bytes();
                let headers = parse_headers(data)?;
                let index = headers.len() - 1;
                // Determine whether the latest_header is on the Ethereum main chain
                // If it is in the main chain, the new header currently needs to be added current_height = latest_height + 1
                // If it is not in the main chain, it means that reorg has occurred, and you need to trace back from latest_height until the back traced header is on the main chain
                lookup_common_ancestor(headers, index, &mut rpc_client);
            }
            None => anyhow::bail!("the bridge cell is not found."),
        }
        Ok(())
    }
}

// Find the common ancestor of the latest header and main chain
fn lookup_common_ancestor(
    headers: Vec<BlockHeader>,
    index: usize,
    rpc_client: &mut Web3Client,
) -> BoxFuture<()> {
    async move {
        let latest_header = &headers[index];
        let block = rpc_client
            .get_block(BlockId::Hash(latest_header.hash.unwrap()))
            .await;
        match block {
            Ok(block) => {
                // The latest header on the ckb contract is on the Ethereum main chain, and goes directly to the normal loop logic
                do_relay_loop(block).await.unwrap();
            }
            Err(_) => {
                // The latest header on ckb is not on the Ethereum main chain and needs to be backtracked
                if index > 1 {
                    lookup_common_ancestor(headers, index - 1, rpc_client).await;
                } else {
                    panic!("system error! can not find the common ancestor with main chain.")
                }
            }
        }
    }
    .boxed()
}

// 查找 latest header 和 main chain 的共同祖先
// pub async fn lookup_common_ancestor(
//     headers: Vec<BlockHeader>,
//     index: usize,
//     rpc_client: &mut Web3Client,
// ) -> Result<()> {
//     let latest_header = &headers[index];
//     let block = rpc_client
//         .get_block(BlockId::Hash(latest_header.hash.unwrap()))
//         .await;
//     match block {
//         Ok(block) => {
//             // ckb contract 上最新的 header 在以太坊 main chain上，直接走正常 loop 逻辑
//             do_relay_loop(block).await?;
//         }
//         Err(_) => {
//             // ckb 上最新的 header 不在以太坊 main chain 上，需要回溯
//             lookup_common_ancestor(headers, index - 1, rpc_client).await?;
//         }
//     }
//     Ok(())
// }

pub async fn do_relay_loop(block: Block<H256>) -> Result<()> {
    let _number = block.number;
    Ok(())
}

pub fn parse_headers(_data: &[u8]) -> Result<Vec<BlockHeader>> {
    todo!()
}
