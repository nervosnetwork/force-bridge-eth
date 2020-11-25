use crate::transfer::to_eth::get_add_ckb_headers_func;
use crate::util::ckb_util::Generator;
use crate::util::config::ForceCliConfig;
use crate::util::eth_util::{convert_eth_address, parse_private_key, Web3Client};
use anyhow::{anyhow, bail, Result};
use ethabi::Token;
use ethereum_types::U256;
use log::info;
use shellexpand::tilde;
use std::time::Duration;
use web3::types::{Bytes, H160, H256};

pub struct CKBRelayer {
    pub contract_addr: H160,
    pub priv_key: H256,
    pub ckb_client: Generator,
    pub web3_client: Web3Client,
    pub gas_price: U256,
}

impl CKBRelayer {
    pub fn new(
        config_path: String,
        network: Option<String>,
        priv_key_path: String,
        gas_price: u64,
    ) -> Result<CKBRelayer> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_cli_config = ForceCliConfig::new(config_path.as_str())?;
        let deployed_contracts = force_cli_config
            .deployed_contracts
            .as_ref()
            .expect("contracts should be deployed");
        let eth_rpc_url = force_cli_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_cli_config.get_ckb_rpc_url(&network)?;
        let ckb_indexer_url = force_cli_config.get_ckb_indexer_url(&network)?;

        let contract_addr = convert_eth_address(&deployed_contracts.eth_ckb_chain_addr)?;
        let ckb_client = Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default())
            .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
        let web3_client = Web3Client::new(eth_rpc_url);
        let gas_price = U256::from(gas_price);
        let priv_key = parse_private_key(&priv_key_path, &force_cli_config, &network)?;

        Ok(CKBRelayer {
            contract_addr,
            priv_key,
            ckb_client,
            web3_client,
            gas_price,
        })
    }
    pub async fn start(&mut self, per_amount: u64) -> Result<()> {
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

        while block_height + per_amount < ckb_current_height {
            let height_range = block_height..block_height + per_amount;
            block_height += per_amount;

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
                self.contract_addr,
                self.priv_key,
                add_headers_abi,
                self.gas_price,
                U256::from(0),
            )
            .await?;
        let tx_receipt = self
            .web3_client
            .client()
            .send_raw_transaction_with_confirmation(Bytes::from(signed_tx), Duration::new(1, 0), 1)
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
