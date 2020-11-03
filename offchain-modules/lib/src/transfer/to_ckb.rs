use crate::util::ckb_util::{ETHSPVProofJson, Generator};
use crate::util::eth_util::Web3Client;
use crate::util::settings::{OutpointConf, ScriptConf, Settings};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_sdk::{AddressPayload, HttpRpcClient, SECP256K1};
use ckb_types::packed::Script;
use ckb_types::prelude::Entity;
use ethabi::{Function, Param, ParamType, Token};
use force_sdk::indexer::IndexerRpcClient;
use force_sdk::tx_helper::{deploy, sign};
use force_sdk::util::{parse_privkey_path, send_tx_sync};
use web3::types::{H160, H256, U256};

pub async fn approve(from: H160, to: H160, url: String, key_path: String) -> Result<H256> {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "approve".to_owned(),
        inputs: vec![
            Param {
                name: "_spender".to_owned(),
                kind: ParamType::Address,
            },
            Param {
                name: "_value".to_owned(),
                kind: ParamType::Uint(256),
            },
        ],
        outputs: vec![Param {
            name: "success".to_owned(),
            kind: ParamType::Bool,
        }],
        constant: false,
    };
    let tokens = [Token::Address(from), Token::Uint(U256::max_value())];
    let input_data = function.encode_input(&tokens)?;
    let res = rpc_client
        .send_transaction(from, to, key_path, input_data, U256::from(0))
        .await?;
    Ok(res)
}

pub async fn lock_token(
    from: H160,
    to: H160,
    url: String,
    key_path: String,
    data: &[Token],
) -> Result<H256> {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "lock".to_owned(),
        inputs: vec![
            Param {
                name: "token".to_owned(),
                kind: ParamType::Address,
            },
            Param {
                name: "amount".to_owned(),
                kind: ParamType::Uint(256),
            },
            Param {
                name: "ckbAddress".to_owned(),
                kind: ParamType::String,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.encode_input(data)?;
    let res = rpc_client
        .send_transaction(from, to, key_path, input_data, U256::from(0))
        .await?;
    Ok(res)
}

pub async fn lock_eth(
    from: H160,
    to: H160,
    url: String,
    key_path: String,
    data: &[Token],
    eth_value: U256,
) -> Result<H256> {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "lockETH".to_owned(),
        inputs: vec![Param {
            name: "ckbAddress".to_owned(),
            kind: ParamType::String,
        }],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.encode_input(data)?;
    let res = rpc_client
        .send_transaction(from, to, key_path, input_data, eth_value)
        .await?;
    Ok(res)
}

pub async fn get_header_rlp(url: String, hash: H256) -> Result<String> {
    let mut rpc_client = Web3Client::new(url);
    Ok(rpc_client.get_header_rlp_with_hash(hash).await?)
}

pub async fn send_eth_spv_proof_tx(
    generator: &mut Generator,
    eth_proof: &ETHSPVProofJson,
    private_key_path: String,
) -> Result<ckb_types::H256> {
    let from_privkey = parse_privkey_path(private_key_path.as_str())?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);

    let unsigned_tx = generator
        .generate_eth_spv_tx(from_lockscript, eth_proof)
        .unwrap();
    let tx = sign(unsigned_tx, &mut generator.rpc_client, &from_privkey).unwrap();
    log::info!(
        "tx: \n{}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .unwrap()
    );
    let tx_hash = send_tx_sync(&mut generator.rpc_client, &tx, 60)
        .map_err(|e| anyhow::anyhow!(e))
        .unwrap();
    let cell_typescript = tx.output(0).unwrap().type_().to_opt();
    let cell_script = match cell_typescript {
        Some(script) => hex::encode(script.as_slice()),
        None => "".to_owned(),
    };
    let print_res = serde_json::json!({
        "tx_hash": hex::encode(tx.hash().as_slice()),
        "cell_typescript": cell_script,
    });
    println!("{}", serde_json::to_string_pretty(&print_res)?);
    Ok(tx_hash)
}

pub fn verify_eth_spv_proof() -> bool {
    todo!()
}

pub fn dev_init(
    config_path: String,
    rpc_url: String,
    indexer_url: String,
    private_key_path: String,
    typescript_path: String,
    lockscript_path: String,
    sudt_path: String,
) -> Result<()> {
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let mut indexer_client = IndexerRpcClient::new(indexer_url);

    let private_key = parse_privkey_path(&private_key_path)?;

    // dev deploy
    let typescript_bin = std::fs::read(typescript_path)?;
    let lockscript_bin = std::fs::read(lockscript_path)?;
    let sudt_bin = std::fs::read(sudt_path)?;
    let typescript_code_hash = blake2b_256(&typescript_bin);
    let typescript_code_hash_hex = hex::encode(&typescript_code_hash);
    let lockscript_code_hash = blake2b_256(&lockscript_bin);
    let lockscript_code_hash_hex = hex::encode(&lockscript_code_hash);
    let sudt_code_hash = blake2b_256(&sudt_bin);
    let sudt_code_hash_hex = hex::encode(&sudt_code_hash);
    let data = vec![typescript_bin, lockscript_bin, sudt_bin];

    let tx = deploy(&mut rpc_client, &mut indexer_client, &private_key, data).unwrap();
    let tx_hash = send_tx_sync(&mut rpc_client, &tx, 60).unwrap();
    let tx_hash_hex = hex::encode(tx_hash.as_bytes());
    let settings = Settings {
        typescript: ScriptConf {
            code_hash: typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 0,
            },
        },
        lockscript: ScriptConf {
            code_hash: lockscript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 1,
            },
        },
        sudt: ScriptConf {
            code_hash: sudt_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex,
                index: 2,
            },
        },
    };
    log::info!("settings: {:?}", &settings);
    settings.write(&config_path).map_err(|e| anyhow!(e))?;
    println!("force-bridge-eth config written to {}", &config_path);
    Ok(())
}
