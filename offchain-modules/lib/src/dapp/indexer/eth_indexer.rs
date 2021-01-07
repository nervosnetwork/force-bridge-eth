use crate::dapp::indexer::db::{
    create_eth_to_ckb_record, get_ckb_to_eth_record_by_eth_hash, get_latest_eth_to_ckb_record,
    is_eth_to_ckb_record_exist, update_ckb_to_eth_record_status, EthToCkbRecord,
};
use crate::transfer::to_ckb::to_eth_spv_proof_json;
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_hex_to_h256, Web3Client};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::Uint128;
use ethabi::{Function, Param, ParamType};
use log::info;
use rusty_receipt_proof_maker::types::UnlockEvent;
use rusty_receipt_proof_maker::{generate_eth_proof, parse_unlock_event, types::EthSpvProof};
use shellexpand::tilde;
use sqlx::MySqlPool;
use std::ops::Add;
use web3::types::U64;

pub struct EthIndexer {
    pub force_config: ForceConfig,
    pub eth_client: Web3Client,
    pub db: MySqlPool,
}

impl EthIndexer {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        db_path: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let eth_client = Web3Client::new(eth_rpc_url);
        let db = MySqlPool::connect(&db_path).await?;
        Ok(EthIndexer {
            force_config,
            eth_client,
            db,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let record_option = get_latest_eth_to_ckb_record(&self.db).await?;
        let mut start_block_number;
        if record_option.is_some() {
            start_block_number = U64::from(record_option.unwrap().block_number);
        } else {
            start_block_number = self.eth_client.client().eth().block_number().await?;
        }

        loop {
            let block = self.eth_client.get_block(start_block_number.into()).await;
            if block.is_err() {
                info!("waiting for new block.");
                tokio::time::delay_for(std::time::Duration::from_secs(3)).await;
                continue;
            }
            let txs = block.unwrap().transactions;
            for tx_hash in txs {
                let hash = hex::encode(tx_hash);
                if !is_eth_to_ckb_record_exist(&self.db, &hash).await? {
                    self.handle_lock_event(hash.clone(), start_block_number.as_u64())
                        .await?;
                }
            }
            start_block_number = start_block_number.add(U64::from(1));
        }
    }

    pub async fn handle_lock_event(&mut self, hash: String, block_number: u64) -> Result<()> {
        let hash_with_0x = format!("{}{}", "0x", hash.clone());
        let (eth_spv_proof, exist) = self.get_eth_spv_proof_with_retry(hash_with_0x.clone(), 3)?;
        if exist {
            let lock_contract_address = self
                .force_config
                .deployed_contracts
                .as_ref()
                .unwrap()
                .eth_token_locker_addr
                .clone();
            let eth_proof_json = to_eth_spv_proof_json(
                hash_with_0x,
                eth_spv_proof.clone(),
                lock_contract_address,
                String::from(self.eth_client.url()),
            )
            .await?;
            let proof_json = serde_json::to_string(&eth_proof_json)?;
            dbg!(proof_json.clone());
            let record = EthToCkbRecord {
                eth_lock_tx_hash: hash.clone(),
                status: "pending".to_string(),
                token_addr: Some(hex::encode(eth_spv_proof.token.as_bytes())),
                ckb_recipient_lockscript: Some(hex::encode(eth_proof_json.recipient_lockscript)),
                locked_amount: Some(Uint128::from(eth_spv_proof.lock_amount).to_string()),
                eth_spv_proof: Some(proof_json),
                replay_resist_outpoint: Some(hex::encode(
                    eth_spv_proof.replay_resist_outpoint.as_slice(),
                )),
                block_number,
                ..Default::default()
            };
            create_eth_to_ckb_record(&self.db, &record).await?;
            info!("create eth_to_ckb record success. tx_hash: {}", hash,);
        } else {
            // if the tx is not lock tx, check if unlock tx.
            self.handle_unlock_event(hash.clone()).await?;
        }
        Ok(())
    }

    pub fn get_eth_spv_proof_with_retry(
        &mut self,
        hash: String,
        max_retry_times: i32,
    ) -> Result<(EthSpvProof, bool)> {
        for retry in 0..max_retry_times {
            let ret = generate_eth_proof(hash.clone(), String::from(self.eth_client.url()).clone());
            match ret {
                Ok(proof) => return Ok((proof, true)),
                Err(e) => {
                    info!(
                        "get eth receipt proof failed, retried {} times, err: {}",
                        retry, e
                    );
                    if e.to_string().contains("the locked tx is not exist") {
                        info!("the locked tx is not exist");
                        return Ok((Default::default(), false));
                    }
                }
            }
        }
        Err(anyhow!(
            "Failed to generate eth proof for lock tx:{}, after retry {} times",
            hash.as_str(),
            max_retry_times
        ))
    }

    pub async fn handle_unlock_event(&mut self, hash: String) -> Result<()> {
        let hash_with_0x = format!("{}{}", "0x", hash.clone());
        let record_op = get_ckb_to_eth_record_by_eth_hash(&self.db, hash.clone()).await?;
        if record_op.is_some() {
            let record = record_op.unwrap();
            if record.status == "success" {
                return Ok(());
            }
        }
        let (_event, exist) = self.parse_unlock_event_with_retry(hash_with_0x.clone(), 3)?;
        if exist {
            let tx_hash = convert_hex_to_h256(&hash)?;
            let tx = self
                .eth_client
                .client()
                .eth()
                .transaction(tx_hash.into())
                .await?;
            if tx.is_some() {
                let input = tx.unwrap().input;
                let function = Function {
                    name: "unlockToken".to_owned(),
                    inputs: vec![
                        Param {
                            name: "ckbTxProof".to_owned(),
                            kind: ParamType::Bytes,
                        },
                        Param {
                            name: "ckbTx".to_owned(),
                            kind: ParamType::Bytes,
                        },
                    ],
                    outputs: vec![],
                    constant: false,
                };
                let input_data = function.decode_input(input.0[4..].as_ref())?;
                let ckb_tx = input_data[1].clone();
                let tx_raw = &ckb_tx.to_bytes();
                if tx_raw.is_some() {
                    let ckb_tx_hash = blake2b_256(tx_raw.as_ref().unwrap().as_slice());
                    let ckb_tx_hash_str = hex::encode(ckb_tx_hash);
                    let ret = update_ckb_to_eth_record_status(
                        &self.db,
                        ckb_tx_hash_str,
                        hash.clone(),
                        "success",
                    )
                    .await?;
                    if !ret {
                        log::error!(
                            "failed to update ckb to eth cross chain record. ckb_tx_hash: {:?}",
                            hash
                        );
                    }
                }
            }
        }
        Ok(())
    }

    pub fn parse_unlock_event_with_retry(
        &mut self,
        hash: String,
        max_retry_times: i32,
    ) -> Result<(UnlockEvent, bool)> {
        for retry in 0..max_retry_times {
            dbg!(hash.clone());
            let ret = parse_unlock_event(hash.clone(), String::from(self.eth_client.url()).clone());
            match ret {
                Ok(event) => return Ok((event, true)),
                Err(e) => {
                    info!(
                        "get eth receipt proof failed, retried {} times, err: {}",
                        retry, e
                    );
                    if e.to_string().contains("the unlocked tx is not exist") {
                        info!("the unlocked tx is not exist");
                        return Ok((Default::default(), false));
                    }
                }
            }
        }
        Err(anyhow!(
            "Failed to parse unlock event for tx:{}, after retry {} times",
            hash.as_str(),
            max_retry_times
        ))
    }
}

#[test]
fn test_decode() {
    let input = "0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000aaaa0000001c0000001e0000002600000046000000660000008600000002003d0000000000000087665cdcc219b392791360a8077fb12e37a43554434f1694026a2ad4ecae078ea4f6038b2f6b634d0aa46cd45be2880cf89153f7866aa7e857f91e4f60da69e584635bda360a131d909a63dd21b4ff2f757edcfbb0e43748520959c37aac910b01000000404bb346c38c5efb4471763d1c7771085bea1c3bd4d9d8509d8f692f8e43e80600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000039d9d0300001c00000020000000b8000000bc00000018010000d50200000000000004000000e287ea8bc97eceaf3420e780b7341c739128da626b1d4158923820727ada5e7a0000000000e287ea8bc97eceaf3420e780b7341c739128da626b1d4158923820727ada5e7a0200000000e287ea8bc97eceaf3420e780b7341c739128da626b1d4158923820727ada5e7a0400000000a777fd1964ffa98a7b0b6c09ff71691705d84d5ed1badfb14271a3a870bdd06b000000000100000000020000000000000000000000fd6edeff40306873d838681e8020aee3ac5080c63a1235de3102767ec6a0d3750000000000000000000000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f810a000000bd01000010000000a60000005c0100009600000010000000180000006100000000ba1dd205000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c35000000100000003000000031000000ed0df97ea89ce848b20479194c9eb50cda612837f2db516b828ffeea61473ff30000000000b600000010000000180000006100000000c817a804000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c55000000100000003000000031000000e1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df4190020000000b5ff94e85f04396cf5b852446eb75d8880cad4d94a1c17d0e5cd70470e6c2ba86100000010000000180000006100000080e4c47b606dc11b490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171cc800000010000000b0000000c40000009c000000403a53a7dfa7a4ab022e53feff11232b3140407d0000000000000000000000000000000000000000cd62e77cfe0386343c15c13528675aae9925d7ae88d9ffc645fef37c2097140cdc2923726d4efe16131e76e85757b446138e39ceda6d3ad483fb11a5619e65035c3139acdb17c26e73647b7f0ac62a4036ca4e720200000000000000000000000000000001000000000000000000000000000000100000005a00000000000000000000000000000000000000000000";
    let input_bin = hex::decode(input).unwrap();
    let function = Function {
        name: "unlockToken".to_owned(),
        inputs: vec![
            Param {
                name: "ckbTxProof".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "ckbTx".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.decode_input(input_bin.as_slice()).unwrap();
    let ckb_tx = input_data[1].clone();
    let tx_raw = &ckb_tx.to_bytes();
    if tx_raw.is_some() {
        let ckb_tx_hash = blake2b_256(tx_raw.as_ref().unwrap().as_slice());
        let hash = hex::encode(ckb_tx_hash);
        assert_eq!(
            hash,
            "a4f6038b2f6b634d0aa46cd45be2880cf89153f7866aa7e857f91e4f60da69e5"
        )
    }
}
