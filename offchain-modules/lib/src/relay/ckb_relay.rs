use crate::transfer::to_eth::{get_add_ckb_headers_func, get_msg_hash, get_msg_signature};
use crate::util::ckb_tx_generator::Generator;
use crate::util::config::CKBRelayMultiSignConf;
use crate::util::eth_util::{convert_eth_address, relay_header_transaction, Web3Client};
use anyhow::{anyhow, Result};
use ethabi::Token;
use ethereum_types::U256;
use futures::future::try_join_all;
use log::info;
use std::ops::Add;
use std::time::Instant;
use web3::types::{H160, H256};

pub struct CKBRelayer {
    pub contract_addr: H160,
    pub priv_key: H256,
    pub ckb_client: Generator,
    pub web3_client: Web3Client,
    pub gas_price: U256,
    pub mutlisig_conf: CKBRelayMultiSignConf,
}

impl CKBRelayer {
    pub fn new(
        ckb_rpc_url: String,
        ckb_indexer_url: String,
        eth_rpc_url: String,
        priv_key: H256,
        eth_ckb_chain_addr: String,
        gas_price: u64,
        mutlisig_conf: CKBRelayMultiSignConf,
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
            mutlisig_conf,
        })
    }

    pub async fn start(
        &mut self,
        eth_url: String,
        per_amount: u64,
        max_tx_amount: u64,
    ) -> Result<()> {
        let mut client_block_number = self
            .web3_client
            .get_contract_height("latestBlockNumber", self.contract_addr)
            .await?;
        let client_init_height = self
            .web3_client
            .get_contract_height("initBlockNumber", self.contract_addr)
            .await?;
        if client_block_number < client_init_height {
            panic!(
                "light client contract state error: latest height  : {}  <  init height : {}",
                client_block_number, client_init_height
            );
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
                .is_header_exist(client_block_number, ckb_header_hash, self.contract_addr)
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
            let results = try_join_all(futures).await?;
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
        let chain_id = self.web3_client.client().eth().chain_id().await?;
        let headers_msg_hash = get_msg_hash(chain_id, self.contract_addr, &headers)?;

        let mut signatures: Vec<u8> = vec![];
        for i in 0..self.mutlisig_conf.threshold {
            let privkey =
                H256::from_slice(hex::decode(self.mutlisig_conf.privkeys[i].clone())?.as_slice());
            let mut signature = get_msg_signature(&headers_msg_hash, privkey)?;
            signatures.append(&mut signature);
        }
        info!("msg signatures {}", hex::encode(&signatures));

        let add_headers_abi =
            add_headers_func.encode_input(&[Token::Bytes(headers), Token::Bytes(signatures)])?;
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
}
