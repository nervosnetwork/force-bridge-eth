use crate::transfer::to_eth::get_add_ckb_headers_func;
use crate::util::ckb_util::Generator;
use crate::util::eth_util::Web3Client;
use anyhow::{anyhow, bail, Result};
use ethabi::Token;
use ethereum_types::U256;
use log::info;
use std::time::Duration;
use web3::types::{Bytes, H160};

pub struct CKBRelayer {
    pub from: H160,
    pub contract_addr: H160,
    pub priv_key_path: String,
    pub ckb_client: Generator,
    pub web3_client: Web3Client,
}

impl CKBRelayer {
    pub fn new(
        ckb_rpc_url: String,
        indexer_url: String,
        eth_rpc_url: String,
        from: H160,
        contract_addr: H160,
        priv_key_path: String,
    ) -> Result<CKBRelayer> {
        let ckb_client = Generator::new(ckb_rpc_url, indexer_url, Default::default())
            .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
        let web3_client = Web3Client::new(eth_rpc_url);

        Ok(CKBRelayer {
            from,
            contract_addr,
            priv_key_path,
            ckb_client,
            web3_client,
        })
    }
    pub async fn start(&mut self) -> Result<()> {
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
        let block_gap = 1;

        let ckb_current_height = self
            .ckb_client
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;

        while block_height < ckb_current_height {
            let height_range = block_height..block_height + block_gap;
            block_height += block_gap;

            let heights: Vec<u64> = height_range.clone().collect();
            self.relay_headers(heights).await?;
        }
        Ok(())
    }

    pub async fn relay_headers(&mut self, heights: Vec<u64>) -> Result<()> {
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
                self.from,
                self.contract_addr,
                self.priv_key_path.clone(),
                add_headers_abi,
                U256::from(0),
            )
            .await?;
        let tx_receipt = self
            .web3_client
            .client()
            .send_raw_transaction_with_confirmation(
                Bytes::from(signed_tx),
                Duration::new(10, 100),
                1,
            )
            .await?;
        let tx_status = tx_receipt
            .status
            .ok_or_else(|| anyhow!("tx receipt is none"))?;
        let hex_tx_hash = hex::encode(tx_receipt.transaction_hash);
        if tx_status.as_usize() == 1 {
            info!("relay headers success. tx_hash : {} ", hex_tx_hash)
        } else {
            bail!(
                "failed to relay headers tx_hash: {} , tx_status : {}",
                hex_tx_hash,
                tx_status
            );
        }
        Ok(())
    }
}
