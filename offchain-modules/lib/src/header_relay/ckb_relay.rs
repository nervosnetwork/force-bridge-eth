use crate::transfer::to_eth::{get_add_ckb_headers_func, get_msg_hash, get_msg_signature};
use crate::util::ckb_proof_helper::CBMT;
use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::covert_to_h256;
use crate::util::config::ForceConfig;
use crate::util::eth_util::{
    convert_eth_address, parse_private_key, parse_secret_key, relay_header_transaction, Web3Client,
};
use crate::util::rocksdb::open_rocksdb;
use anyhow::{anyhow, bail, Result};
use ckb_sdk::HttpRpcClient;
use ethabi::Token;
use ethereum_types::U256;
use force_eth_types::eth_recipient_cell::ETHRecipientDataView;
use force_sdk::constants::{
    BURN_TX_MAX_NUM, BURN_TX_MAX_WAITING_BLOCKS, MAINNET_CKB_WAITING_BLOCKS,
    TESTNET_CKB_WAITING_BLOCKS,
};
use log::info;
use rocksdb::ops::{Get, Put};
use secp256k1::SecretKey;
use std::time::Instant;
use web3::types::{CallRequest, H160, H256};

pub struct CKBRelayer {
    pub contract_addr: H160,
    pub priv_key: H256,
    pub network: String,
    pub ckb_client: Generator,
    pub web3_client: Web3Client,
    pub gas_price: U256,
    pub multisig_privkeys: Vec<SecretKey>,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub ckb_init_height: u64,
    pub db_path: String,
    pub last_burn_tx_height: u64,
    pub last_submit_height: u64,
    pub waiting_burn_txs_count: u64,
    pub confirm: u64,
}

impl CKBRelayer {
    pub fn new(
        config_path: String,
        network: Option<String>,
        priv_key_path: String,
        multisig_privkeys: Vec<String>,
        gas_price: u64,
        confirm: u64,
    ) -> Result<CKBRelayer> {
        let force_config = ForceConfig::new(config_path.as_str())?;
        let deployed_contracts = force_config
            .deployed_contracts
            .as_ref()
            .ok_or_else(|| anyhow!("contracts should be deployed"))?;

        if multisig_privkeys.len() < deployed_contracts.ckb_relay_mutlisig_threshold.threshold {
            bail!(
                "the mutlisig privkeys number is less. expect {}, actual {} ",
                deployed_contracts.ckb_relay_mutlisig_threshold.threshold,
                multisig_privkeys.len()
            );
        }

        let net = match network.clone() {
            Some(v) => v,
            None => String::default(),
        };

        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let priv_key = parse_private_key(&priv_key_path, &force_config, &network)?;
        let multisig_privkeys = multisig_privkeys
            .into_iter()
            .map(|k| parse_private_key(&k, &force_config, &network))
            .collect::<Result<Vec<H256>>>()?;

        let contract_addr = convert_eth_address(&deployed_contracts.eth_ckb_chain_addr)?;
        let mut ckb_client =
            Generator::new(ckb_rpc_url.clone(), ckb_indexer_url, Default::default())
                .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
        let web3_client = Web3Client::new(eth_rpc_url.clone());
        let gas_price = U256::from(gas_price);

        let ckb_init_height = CKBRelayer::get_ckb_contract_deloy_height(
            &mut ckb_client,
            deployed_contracts
                .recipient_typescript
                .outpoint
                .tx_hash
                .clone(),
        )?;

        let db_path = force_config.ckb_rocksdb_path;
        let last_burn_tx_height = 0;
        let last_submit_height = 0;
        let waiting_burn_txs_count = 0;

        Ok(CKBRelayer {
            ckb_rpc_url,
            eth_rpc_url,
            ckb_init_height,
            db_path,
            last_burn_tx_height,
            last_submit_height,
            waiting_burn_txs_count,
            contract_addr,
            priv_key,
            ckb_client,
            web3_client,
            gas_price,
            confirm,
            network: net,
            multisig_privkeys: multisig_privkeys
                .iter()
                .map(|&privkey| parse_secret_key(privkey))
                .collect::<Result<Vec<SecretKey>>>()?,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let ckb_current_height = self
            .ckb_client
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;

        self.store_history_transaction_root(
            self.ckb_init_height,
            ckb_current_height,
            self.db_path.clone(),
        )?;

        if ckb_current_height <= self.confirm {
            info!(
                "ckb_current_height {:?} not reach confirm: {:?}",
                ckb_current_height, self.confirm
            );
            return Ok(());
        }
        let confirmed_height = ckb_current_height - self.confirm;
        let waiting_blocks = match self.network.as_str() {
            "mainnet" => MAINNET_CKB_WAITING_BLOCKS,
            _ => TESTNET_CKB_WAITING_BLOCKS,
        };
        // usually relay every 5000 blocks
        // 20 burn txs will trigger relay
        // burn txs can not wait over 100 blocks
        if confirmed_height - self.last_submit_height > waiting_blocks
            || (self.waiting_burn_txs_count >= BURN_TX_MAX_NUM
                && confirmed_height >= self.last_burn_tx_height)
            || (confirmed_height - self.last_burn_tx_height > BURN_TX_MAX_WAITING_BLOCKS
                && self.waiting_burn_txs_count > 0)
        {
            let merkle_root = self.get_history_merkle_root(
                self.ckb_init_height,
                confirmed_height,
                self.db_path.clone(),
            )?;
            let nonce = self.web3_client.get_eth_nonce(&self.priv_key).await?;
            let sign_tx = self
                .relay_headers(self.ckb_init_height, confirmed_height, merkle_root, nonce)
                .await?;
            let task_future = relay_header_transaction(self.eth_rpc_url.clone(), sign_tx);
            let timeout_future = tokio::time::delay_for(std::time::Duration::from_secs(1800));
            let now = Instant::now();
            tokio::select! {
                v = task_future => { v?; }
                _ = timeout_future => {
                    bail!("relay headers timeout");
                }
            }
            self.last_submit_height = confirmed_height;
            self.waiting_burn_txs_count = 0;
            info!("relay headers time elapsed: {:?}", now.elapsed());
        }

        Ok(())
    }

    pub async fn relay_headers(
        &mut self,
        init_block_number: u64,
        latest_block_number: u64,
        history_tx_root: [u8; 32],
        asec_nonce: U256,
    ) -> Result<Vec<u8>> {
        info!("relay headers. init_block_number: {:?}, latest_block_number: {:?}, history_tx_root: {:?}, asec_nonce: {:?}",
            init_block_number,
            latest_block_number,
            &history_tx_root,
            &asec_nonce,
        );
        let add_headers_func = get_add_ckb_headers_func();
        let chain_id = self.web3_client.client().eth().chain_id().await?;

        let headers_msg_hash = get_msg_hash(
            chain_id,
            self.contract_addr,
            init_block_number,
            latest_block_number,
            history_tx_root,
        )?;

        let mut signatures: Vec<u8> = vec![];
        for &privkey in self.multisig_privkeys.iter() {
            let mut signature = get_msg_signature(&headers_msg_hash, privkey)?;
            signatures.append(&mut signature);
        }
        info!("msg signatures {}", hex::encode(&signatures));

        let add_headers_abi = add_headers_func.encode_input(&[
            Token::Uint(init_block_number.into()),
            Token::Uint(latest_block_number.into()),
            Token::FixedBytes(history_tx_root.to_vec()),
            Token::Bytes(signatures),
        ])?;
        let gas_price = self.web3_client.client().eth().gas_price().await?.as_u128();

        let request = CallRequest {
            from: None,
            to: Some(self.contract_addr),
            gas: None,
            gas_price: None,
            value: Some(0x0.into()),
            data: Some(add_headers_abi.to_vec().into()),
        };
        let gas_limit = self
            .web3_client
            .client()
            .eth()
            .estimate_gas(request, None)
            .await?
            .as_u128();
        let signed_tx = self
            .web3_client
            .build_sign_tx(
                self.contract_addr,
                self.priv_key,
                add_headers_abi,
                U256::from(gas_price),
                Some(U256::from(gas_limit)),
                U256::from(0u64),
                asec_nonce,
            )
            .await?;
        Ok(signed_tx)
    }

    pub fn get_ckb_contract_deloy_height(
        ckb_client: &mut Generator,
        tx_hash: String,
    ) -> Result<u64> {
        let hash = covert_to_h256(&tx_hash)?;

        let block_hash = ckb_client
            .rpc_client
            .get_transaction(hash)
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("failed to get block height : tx is none"))?
            .tx_status
            .block_hash
            .ok_or_else(|| anyhow!("failed to get block height : block hash is none"))?;

        let ckb_height = ckb_client
            .rpc_client
            .get_block(block_hash)
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("failed to get block height : block is none"))?
            .header
            .inner
            .number;
        Ok(ckb_height)
    }

    pub fn get_history_merkle_root(
        &self,
        start_height: u64,
        latest_height: u64,
        db_path: String,
    ) -> Result<[u8; 32]> {
        let db = open_rocksdb(db_path);

        let mut all_tx_roots = vec![];
        for number in start_height..=latest_height {
            let db_root = db
                .get(number.to_le_bytes())
                .map_err(|err| anyhow!(err))?
                .ok_or_else(|| anyhow!("db ckb root should not be none"))?;
            let mut db_root_raw = [0u8; 32];
            db_root_raw.copy_from_slice(db_root.as_ref());
            all_tx_roots.push(db_root_raw);
        }
        Ok(CBMT::build_merkle_root(&all_tx_roots))
    }

    pub fn store_history_transaction_root(
        &mut self,
        start_height: u64,
        latest_height: u64,
        db_path: String,
    ) -> Result<()> {
        let db = open_rocksdb(db_path);
        let mut rpc_client = HttpRpcClient::new(self.ckb_rpc_url.clone());

        let mut index = latest_height;
        while index >= start_height {
            match rpc_client
                .get_block_by_number(index)
                .map_err(|e| anyhow!("get_header_by_number err: {:?}", e))?
            {
                Some(block_view) => {
                    for tx in block_view.transactions {
                        if tx.inner.outputs_data.is_empty() {
                            continue;
                        }
                        let output_data = tx.inner.outputs_data[0].as_bytes();
                        if ETHRecipientDataView::new(&output_data).is_ok() {
                            self.last_burn_tx_height = index;
                            self.waiting_burn_txs_count += 1;
                            break;
                        }
                    }

                    let header_view = block_view.header;

                    let chain_root = header_view.inner.transactions_root.0;

                    let db_root_option = db.get(index.to_le_bytes()).map_err(|err| anyhow!(err))?;

                    let db_root = match db_root_option {
                        Some(v) => {
                            let mut db_root_raw = [0u8; 32];
                            db_root_raw.copy_from_slice(v.as_ref());
                            db_root_raw
                        }
                        None => [0u8; 32],
                    };

                    if chain_root.to_vec() != db_root {
                        db.put(index.to_le_bytes(), chain_root.to_vec())
                            .map_err(|err| anyhow!(err))?;
                    } else {
                        break;
                    }
                    index -= 1;
                }
                None => {
                    bail!(
                        "cannot get the block transactions root, block_number = {}",
                        index
                    );
                }
            }
        }
        info!(
            "store ckb headers from {:?} to {:?}",
            index + 1,
            latest_height
        );
        Ok(())
    }
}
