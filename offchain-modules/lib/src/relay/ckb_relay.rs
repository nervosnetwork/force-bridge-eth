use crate::transfer::to_eth::get_add_ckb_headers_func;
use crate::util::ckb_util::Generator;
use crate::util::eth_util::{relay_header_transaction, Web3Client};
use anyhow::{anyhow, bail, Result};
use ethabi::Token;
use ethereum_types::U256;
use futures::future::join_all;
use log::info;
use std::ops::Add;
use std::time::Instant;
use web3::types::H160;

pub struct CKBRelayer {
    pub contract_addr: H160,
    pub priv_key_path: String,
    pub ckb_client: Generator,
    pub web3_client: Web3Client,
    pub gas_price: U256,
}

impl CKBRelayer {
    pub fn new(
        ckb_rpc_url: String,
        indexer_url: String,
        eth_rpc_url: String,
        contract_addr: H160,
        priv_key_path: String,
        gas_price: u64,
    ) -> Result<CKBRelayer> {
        let ckb_client = Generator::new(ckb_rpc_url, indexer_url, Default::default())
            .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
        let web3_client = Web3Client::new(eth_rpc_url);
        let gas_price = U256::from(gas_price);

        Ok(CKBRelayer {
            contract_addr,
            priv_key_path,
            ckb_client,
            web3_client,
            gas_price,
        })
    }
    pub async fn start(&mut self, eth_url: String, per_amount: u64) -> Result<()> {
        let mut client_block_number = self
            .web3_client
            .get_contract_height("latestBlockNumber", self.contract_addr)
            .await?;
        let client_init_height = self
            .web3_client
            .get_contract_height("initBlockNumber", self.contract_addr)
            .await?;
        if client_block_number < client_init_height {
            bail!(
                "contract current height  : {}  <  init height : {}",
                client_block_number,
                client_init_height
            );
        }
        while client_block_number > client_init_height {
            let ckb_header_hash = self
                .ckb_client
                .rpc_client
                .get_block_hash(client_block_number)
                .map_err(|e| anyhow!("failed to get block hash: {}", e))?
                .ok_or_else(|| anyhow!("block {:?}  hash is none", client_block_number))?;

            if self
                .web3_client
                .is_header_exist(client_block_number, ckb_header_hash, self.contract_addr)
                .await?
            {
                break;
            }
            client_block_number -= 1;
        }

        let mut block_height = client_block_number + 1;

        let ckb_current_height = self
            .ckb_client
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;
        info!("ckb_current_height:{:?}", ckb_current_height);

        let nonce = self
            .web3_client
            .get_eth_nonce(self.priv_key_path.clone())
            .await?;
        let mut sequence: u64 = 0;

        let mut futures = vec![];
        while block_height + per_amount < ckb_current_height {
            let height_range = block_height..block_height + per_amount;
            block_height += per_amount;

            let heights: Vec<u64> = height_range.clone().collect();
            let sign_tx = self.relay_headers(heights, nonce.add(sequence)).await?;
            futures.push(relay_header_transaction(eth_url.clone(), sign_tx));
            sequence += 1;
        }
        if !futures.is_empty() {
            let now = Instant::now();
            let results = join_all(futures).await;
            info!("join_all execute result {:?}", results);
            info!("relay headers time elapsed: {:?}", now.elapsed());
        }
        Ok(())
    }

    pub async fn relay_headers(&mut self, heights: Vec<u64>, asec_nonce: U256) -> Result<Vec<u8>> {
        let headers = self.ckb_client.get_ckb_headers(heights.clone())?;
        info!(
            "the headers vec of {:?} is {:?} ",
            heights.as_slice(),
            hex::encode(headers.as_slice())
        );

        let add_headers_func = get_add_ckb_headers_func();
        let add_headers_abi = add_headers_func.encode_input(&[Token::Bytes(headers)])?;
        let signed_tx = self
            .web3_client
            .build_sign_tx(
                self.contract_addr,
                self.priv_key_path.clone(),
                add_headers_abi,
                self.gas_price,
                U256::from(0),
                asec_nonce,
            )
            .await?;
        Ok(signed_tx)
    }
}
