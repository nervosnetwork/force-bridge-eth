use crate::transfer::to_eth::{get_add_ckb_headers_func, get_msg_hash, get_msg_signature};
use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::covert_to_h256;
use crate::util::eth_util::{
    convert_eth_address, parse_secret_key, relay_header_transaction, Web3Client,
};
use anyhow::{anyhow, bail, Result};
use ethabi::Token;
use ethereum_types::U256;
use futures::future::try_join_all;
use log::info;
use secp256k1::SecretKey;
use std::ops::Add;
use std::time::Instant;
use web3::types::{H160, H256};

pub struct CKBRelayer {
    pub contract_addr: H160,
    pub priv_key: H256,
    pub ckb_client: Generator,
    pub web3_client: Web3Client,
    pub gas_price: U256,
    pub multisig_privkeys: Vec<SecretKey>,
}

impl CKBRelayer {
    pub fn new(
        ckb_rpc_url: String,
        ckb_indexer_url: String,
        eth_rpc_url: String,
        priv_key: H256,
        eth_ckb_chain_addr: String,
        gas_price: u64,
        multisig_privkeys: Vec<H256>,
    ) -> Result<CKBRelayer> {
        let contract_addr = convert_eth_address(&eth_ckb_chain_addr)?;
        let ckb_client = Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default())
            .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
        let web3_client = Web3Client::new(eth_rpc_url);
        let gas_price = U256::from(gas_price);

        Ok(CKBRelayer {
            contract_addr,
            priv_key,
            ckb_client,
            web3_client,
            gas_price,
            multisig_privkeys: multisig_privkeys
                .iter()
                .map(|&privkey| parse_secret_key(privkey))
                .collect::<Result<Vec<SecretKey>>>()?,
        })
    }

    pub async fn start(
        &mut self,
        eth_url: String,
        per_amount: u64,
        max_tx_amount: u64,
        client_init_height: u64,
    ) -> Result<()> {
        let mut client_block_number = self
            .web3_client
            .get_contract_height("latestBlockNumber", self.contract_addr)
            .await?;
        if client_block_number < client_init_height {
            client_block_number = client_init_height;
        }
        while client_block_number > client_init_height {
            let ckb_header_hash = self
                .ckb_client
                .rpc_client
                .get_block_hash(client_block_number)
                .map_err(|e| anyhow!("failed to get ckb block hash: {}", e))?
                .ok_or_else(|| anyhow!("ckb block {:?}  hash is none", client_block_number))?;

            if self
                .web3_client
                .is_header_exist_v2(client_block_number, ckb_header_hash, self.contract_addr)
                .await?
            {
                break;
            }
            info!(
                "client contract forked, forked block height: {}",
                client_block_number
            );
            client_block_number -= 1;
        }

        let mut block_height = client_block_number + 1;
        let ckb_current_height = self
            .ckb_client
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;
        info!("ckb_current_height:{:?}", ckb_current_height);
        let nonce = self.web3_client.get_eth_nonce(&self.priv_key).await?;
        let mut sequence: u64 = 0;

        let mut futures = vec![];

        let mut target_height = block_height + per_amount * max_tx_amount;
        if target_height > ckb_current_height {
            target_height = ckb_current_height;
        }

        while block_height + per_amount <= target_height {
            let height_range = block_height..block_height + per_amount;
            block_height += per_amount;

            let heights: Vec<u64> = height_range.clone().collect();
            let sign_tx = self.relay_headers(heights, nonce.add(sequence)).await?;
            futures.push(relay_header_transaction(eth_url.clone(), sign_tx));
            sequence += 1;
        }
        if !futures.is_empty() {
            let now = Instant::now();
            let timeout_future = tokio::time::delay_for(std::time::Duration::from_secs(1800));
            let task_future = try_join_all(futures);
            tokio::select! {
                v = task_future => { v?; }
                _ = timeout_future => {
                    bail!("relay headers timeout");
                }
            }
            info!("relay headers time elapsed: {:?}", now.elapsed());
        }
        Ok(())
    }

    pub async fn relay_headers(&mut self, heights: Vec<u64>, asec_nonce: U256) -> Result<Vec<u8>> {
        let headers = self.ckb_client.get_ckb_headers_v2(heights.clone())?;

        let add_headers_func = get_add_ckb_headers_func();
        let chain_id = self.web3_client.client().eth().chain_id().await?;

        let header_datas = headers
            .into_iter()
            .map(|header| Token::Bytes(header))
            .collect::<Vec<Token>>();

        let headers_msg_hash = get_msg_hash(chain_id, self.contract_addr, header_datas.clone())?;

        let mut signatures: Vec<u8> = vec![];
        for &privkey in self.multisig_privkeys.iter() {
            let mut signature = get_msg_signature(&headers_msg_hash, privkey)?;
            signatures.append(&mut signature);
        }
        info!("msg signatures {}", hex::encode(&signatures));

        let add_headers_abi = add_headers_func
            .encode_input(&[Token::Array(header_datas), Token::Bytes(signatures)])?;
        let increased_gas_price =
            self.web3_client.client().eth().gas_price().await?.as_u128() * 3 / 2;
        let signed_tx = self
            .web3_client
            .build_sign_tx(
                self.contract_addr,
                self.priv_key,
                add_headers_abi,
                U256::from(increased_gas_price),
                Some(U256::from(1_500_000)),
                U256::from(0),
                asec_nonce,
            )
            .await?;
        Ok(signed_tx)
    }

    pub fn get_ckb_contract_deloy_height(&mut self, tx_hash: String) -> Result<u64> {
        let hash = covert_to_h256(&tx_hash)?;

        let block_hash = self
            .ckb_client
            .rpc_client
            .get_transaction(hash.clone())
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("failed to get block height : tx is none"))?
            .tx_status
            .block_hash
            .ok_or_else(|| anyhow!("failed to get block height : block hash is none"))?;

        let ckb_height = self
            .ckb_client
            .rpc_client
            .get_block(block_hash)
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("failed to get block height : block is none"))?
            .header
            .inner
            .number;
        Ok(ckb_height)
    }
}
