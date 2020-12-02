use crate::util::ckb_util::{
    build_lockscript_from_address, get_secret_key, parse_privkey, parse_privkey_path,
    ETHSPVProofJson, Generator,
};
use crate::util::config::{CellScript, DeployedContracts, ForceConfig, OutpointConf, ScriptConf};
use crate::util::eth_proof_helper::{read_roots_collection_raw, RootsCollectionJson};
use crate::util::eth_util::{
    build_lock_eth_payload, build_lock_token_payload, convert_eth_address, parse_private_key,
    Web3Client,
};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_sdk::{Address, AddressPayload, GenesisInfo, HttpRpcClient, HumanCapacity, SECP256K1};
use ckb_types::core::{BlockView, TransactionView};
use ckb_types::packed::{Byte32, CellOutput, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use cmd_lib::run_fun;
use ethabi::{Function, Param, ParamType, Token};
use force_eth_types::generated::basic::ETHAddress;
use force_eth_types::generated::eth_bridge_lock_cell::ETHBridgeLockArgs;
use force_eth_types::generated::eth_header_cell::DagsMerkleRoots;
use force_sdk::indexer::IndexerRpcClient;
use force_sdk::tx_helper::{sign, TxHelper};
use force_sdk::util::{ensure_indexer_sync, send_tx_sync};
use log::info;
use rusty_receipt_proof_maker::generate_eth_proof;
use secp256k1::SecretKey;
use serde_json::Value;
use shellexpand::tilde;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use web3::types::{H160, H256, U256};

pub async fn approve(
    config_path: String,
    network: Option<String>,
    key_path: String,
    erc20_addr: String,
    gas_price: u64,
    wait: bool,
) -> Result<H256> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let url = force_config.get_ethereum_rpc_url(&network)?;
    let approve_recipient = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    let to = convert_eth_address(&erc20_addr)?;
    let eth_private_key = parse_private_key(&key_path, &force_config, &network)?;

    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "approve".to_owned(),
        inputs: vec![
            Param {
                name: "spender".to_owned(),
                kind: ParamType::Address,
            },
            Param {
                name: "value".to_owned(),
                kind: ParamType::Uint(256),
            },
        ],
        outputs: vec![Param {
            name: "success".to_owned(),
            kind: ParamType::Bool,
        }],
        constant: false,
    };
    let tokens = [
        Token::Address(approve_recipient),
        Token::Uint(U256::max_value()),
    ];
    let input_data = function.encode_input(&tokens)?;
    let res = rpc_client
        .send_transaction(
            to,
            eth_private_key,
            input_data,
            U256::from(gas_price),
            U256::from(0),
            wait,
        )
        .await?;
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub async fn lock_token(
    config_path: String,
    network: Option<String>,
    key_path: String,
    token: String,
    ckb_recipient_address: String,
    amount: u128,
    bridge_fee: u128,
    sudt_extra_data: String,
    replay_resist_outpoint: String,
    gas_price: u64,
    wait: bool,
) -> Result<H256> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let to = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    let token_addr = convert_eth_address(&token)?;
    let recipient_lockscript = build_lockscript_from_address(ckb_recipient_address.as_str())?;
    let data = vec![
        Token::Address(token_addr),
        Token::Uint(U256::from(amount)),
        Token::Uint(U256::from(bridge_fee)),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(hex::decode(replay_resist_outpoint)?),
        Token::Bytes(sudt_extra_data.as_bytes().to_vec()),
    ];

    let mut rpc_client = Web3Client::new(ethereum_rpc_url);
    let input_data = build_lock_token_payload(data.as_slice())?;

    let res = rpc_client
        .send_transaction(
            to,
            parse_private_key(key_path.as_str(), &force_config, &network)?,
            input_data,
            U256::from(gas_price),
            U256::from(0),
            wait,
        )
        .await?;
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub async fn lock_eth(
    config_path: String,
    network: Option<String>,
    key_path: String,
    ckb_recipient_address: String,
    amount: u128,
    bridge_fee: u128,
    sudt_extra_data: String,
    replay_resist_outpoint: String,
    gas_price: u64,
    wait: bool,
) -> Result<H256> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let to = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    let recipient_lockscript = build_lockscript_from_address(ckb_recipient_address.as_str())?;

    let data = vec![
        Token::Uint(U256::from(bridge_fee)),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(hex::decode(replay_resist_outpoint)?),
        Token::Bytes(sudt_extra_data.as_bytes().to_vec()),
    ];
    let mut rpc_client = Web3Client::new(ethereum_rpc_url);
    let input_data = build_lock_eth_payload(data.as_slice())?;
    let res = rpc_client
        .send_transaction(
            to,
            parse_private_key(key_path.as_str(), &force_config, &network)?,
            input_data,
            U256::from(gas_price),
            U256::from(amount),
            wait,
        )
        .await?;
    Ok(res)
}

pub async fn get_header_rlp(url: String, hash: H256) -> Result<String> {
    let mut rpc_client = Web3Client::new(url);
    Ok(rpc_client.get_header_rlp(hash.into()).await?)
}

pub async fn generate_eth_spv_proof_json(
    hash: String,
    ethereum_rpc_url: String,
    eth_token_locker_addr: String,
) -> Result<ETHSPVProofJson> {
    let eth_spv_proof = generate_eth_proof(hash.clone(), ethereum_rpc_url.clone())
        .map_err(|e| anyhow!("Failed to generate eth proof. {:?}", e))?;
    let header_rlp = get_header_rlp(ethereum_rpc_url.clone(), eth_spv_proof.block_hash).await?;
    info!("eth_spv_proof: {:?}", eth_spv_proof);
    let hash_str = hash.clone();
    let log_index = eth_spv_proof.log_index;
    let network = ethereum_rpc_url;
    let proof_hex = run_fun! {
    node eth-proof/index.js proof --hash ${hash_str} --index ${log_index} --network ${network}}
    .unwrap();
    let proof_json: Value = serde_json::from_str(&proof_hex).unwrap();
    info!("generate proof json: {:?}", proof_json);
    // TODO: refactor to parse with static struct instead of dynamic parsing
    let mut proof_vec = vec![];
    for item in proof_json["proof"].as_array().unwrap() {
        proof_vec.push(item.as_str().unwrap().to_owned());
    }
    Ok(ETHSPVProofJson {
        log_index: u64::try_from(log_index).unwrap(),
        log_entry_data: String::from(proof_json["log_data"].as_str().unwrap()),
        receipt_index: eth_spv_proof.receipt_index,
        receipt_data: String::from(proof_json["receipt_data"].as_str().unwrap()),
        header_data: header_rlp,
        proof: proof_vec,
        token: eth_spv_proof.token,
        lock_amount: eth_spv_proof.lock_amount,
        recipient_lockscript: eth_spv_proof.recipient_lockscript,
        sudt_extra_data: eth_spv_proof.sudt_extra_data,
        bridge_fee: eth_spv_proof.bridge_fee,
        replay_resist_outpoint: eth_spv_proof.replay_resist_outpoint,
        eth_address: convert_eth_address(&eth_token_locker_addr)?,
    })
}

pub async fn send_eth_spv_proof_tx(
    generator: &mut Generator,
    config_path: String,
    eth_proof: &ETHSPVProofJson,
    from_privkey: SecretKey,
) -> Result<ckb_types::H256> {
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);

    let unsigned_tx = generator.generate_eth_spv_tx(config_path, from_lockscript, eth_proof)?;
    let tx =
        sign(unsigned_tx, &mut generator.rpc_client, &from_privkey).map_err(|err| anyhow!(err))?;
    log::info!(
        "tx: \n{}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .map_err(|err| anyhow!(err))?
    );
    let tx_hash = send_tx_sync(&mut generator.rpc_client, &tx, 120)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let cell_typescript = tx
        .output(0)
        .ok_or_else(|| anyhow!("no out_put found"))?
        .type_()
        .to_opt();
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

#[allow(clippy::too_many_arguments)]
pub async fn deploy_ckb(
    config_path: String,
    network: Option<String>,
    eth_dag_path: Option<String>,
) -> Result<()> {
    let config_path = tilde(config_path.as_str()).into_owned();
    let mut force_config = ForceConfig::new(config_path.as_str())?;
    let rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let indexer_url = force_config.get_ckb_indexer_url(&network)?;
    let ckb_private_keys = force_config.get_ckb_private_keys(&network)?;
    let eth_dag_path = if let Some(dag_path) = eth_dag_path.as_ref() {
        dag_path
    } else {
        &force_config.eth_dag_path
    };
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let mut indexer_client = IndexerRpcClient::new(indexer_url);
    let private_key = get_secret_key(&ckb_private_keys[0].as_str())?;
    let bridge_typescript_path = force_config.get_bridge_typescript_bin_path()?;
    let bridge_lockscript_path = force_config.get_bridge_lockscript_bin_path()?;
    let light_client_typescript_path = force_config.get_light_client_typescript_bin_path()?;
    let light_client_lockscript_path = force_config.get_light_client_lockscript_bin_path()?;
    let recipient_typescript_path = force_config.get_recipient_typescript_bin_path()?;
    let sudt_path = force_config.get_sudt_typescript_bin_path()?;

    // dev deploy
    let bridge_typescript_bin = std::fs::read(bridge_typescript_path)?;
    let bridge_lockscript_bin = std::fs::read(bridge_lockscript_path)?;
    let light_client_typescript_bin = std::fs::read(light_client_typescript_path)?;
    let light_client_lockscript_bin = std::fs::read(light_client_lockscript_path)?;
    let recipient_typescript_bin = std::fs::read(recipient_typescript_path)?;
    let sudt_bin = std::fs::read(sudt_path)?;

    let bridge_typescript_code_hash = blake2b_256(&bridge_typescript_bin);
    let bridge_typescript_code_hash_hex = hex::encode(&bridge_typescript_code_hash);

    let light_client_typescript_code_hash = blake2b_256(&light_client_typescript_bin);
    let light_client_typescript_code_hash_hex = hex::encode(&light_client_typescript_code_hash);

    let light_client_lockscript_code_hash = blake2b_256(&light_client_lockscript_bin);
    let light_client_lockscript_code_hash_hex = hex::encode(&light_client_lockscript_code_hash);

    let bridge_lockscript_code_hash = blake2b_256(&bridge_lockscript_bin);
    let bridge_lockscript_code_hash_hex = hex::encode(&bridge_lockscript_code_hash);

    let recipient_typescript_code_hash = blake2b_256(&recipient_typescript_bin);
    let recipient_typescript_code_hash_hex = hex::encode(&recipient_typescript_code_hash);

    let sudt_code_hash = blake2b_256(&sudt_bin);
    let sudt_code_hash_hex = hex::encode(&sudt_code_hash);

    let data = vec![
        bridge_lockscript_bin,
        bridge_typescript_bin,
        light_client_typescript_bin,
        light_client_lockscript_bin,
        recipient_typescript_bin,
        sudt_bin,
    ];

    let tx = deploy(
        &mut rpc_client,
        &mut indexer_client,
        &private_key,
        data,
        eth_dag_path.as_str(),
    )
    .map_err(|err| anyhow!(err))?;
    let tx_hash = send_tx_sync(&mut rpc_client, &tx, 60)
        .await
        .map_err(|err| anyhow!(err))?;
    let tx_hash_hex = hex::encode(tx_hash.as_bytes());

    let deployed_contracts = DeployedContracts {
        bridge_lockscript: ScriptConf {
            code_hash: bridge_lockscript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 0,
                dep_type: 0,
            },
        },
        bridge_typescript: ScriptConf {
            code_hash: bridge_typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 1,
                dep_type: 0,
            },
        },
        light_client_typescript: ScriptConf {
            code_hash: light_client_typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 2,
                dep_type: 0,
            },
        },
        light_client_lockscript: ScriptConf {
            code_hash: light_client_lockscript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 3,
                dep_type: 0,
            },
        },
        recipient_typescript: ScriptConf {
            code_hash: recipient_typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 4,
                dep_type: 0,
            },
        },
        sudt: ScriptConf {
            code_hash: sudt_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 5,
                dep_type: 0,
            },
        },
        dag_merkle_roots: OutpointConf {
            tx_hash: tx_hash_hex,
            index: 6,
            dep_type: 0,
        },
        light_client_cell_script: CellScript {
            cell_script: "".to_string(),
        },
        ..Default::default()
    };
    log::info!("ckb_scripts: {:?}", &deployed_contracts);
    force_config.deployed_contracts = Some(deployed_contracts);
    force_config.write(&config_path)?;
    println!("force-bridge config written to {}", &config_path);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_bridge_cell(
    config_path: String,
    network: Option<String>,
    private_key_path: String,
    tx_fee: String,
    capacity: String,
    eth_token_address_str: String,
    recipient_address: String,
    bridge_fee: u128,
    cell_num: u32,
) -> Result<Vec<String>> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let indexer_url = force_config.get_ckb_indexer_url(&network)?;
    let mut generator = Generator::new(rpc_url, indexer_url, deployed_contracts.clone())
        .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .await
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
    let from_privkey = parse_privkey_path(&private_key_path, &force_config, &network)?;
    let from_lockscript = parse_privkey(&from_privkey);

    let tx_fee: u64 = HumanCapacity::from_str(&tx_fee)
        .map_err(|e| anyhow!(e))?
        .into();
    let capacity: u64 = HumanCapacity::from_str(&capacity)
        .map_err(|e| anyhow!(e))?
        .into();
    let eth_contract_address =
        convert_eth_address(deployed_contracts.eth_token_locker_addr.as_str())?;
    let eth_token_address = convert_eth_address(eth_token_address_str.as_str())?;
    let recipient_lockscript = Script::from(
        Address::from_str(&recipient_address)
            .map_err(|err| anyhow!("invalid recipient address: {}", err))?
            .payload(),
    );
    let unsigned_tx = generator
        .create_bridge_cell(
            tx_fee,
            capacity,
            from_lockscript,
            eth_token_address,
            eth_contract_address,
            recipient_lockscript,
            bridge_fee,
            cell_num,
        )
        .map_err(|e| anyhow!("failed to build burn tx : {}", e))?;
    let tx = sign(unsigned_tx, &mut generator.rpc_client, &from_privkey)
        .map_err(|e| anyhow!("sign error {}", e))?;
    log::info!(
        "tx: \n{}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .map_err(|err| anyhow!(err))?
    );
    let tx_hash = send_tx_sync(&mut generator.rpc_client, &tx, 60)
        .await
        .map_err(|err| anyhow!(err))?;
    let mut res = vec![];
    for i in 0..cell_num {
        let outpoint = OutPoint::new_builder()
            .tx_hash(Byte32::from_slice(tx_hash.as_ref())?)
            .index(i.pack())
            .build();
        let outpoint_hex = hex::encode(outpoint.as_slice());
        res.push(outpoint_hex);
    }
    Ok(res)
}

pub fn build_eth_bridge_lock_args(
    eth_token_address: H160,
    eth_contract_address: H160,
) -> Result<ETHBridgeLockArgs> {
    let args = ETHBridgeLockArgs::new_builder()
        .eth_token_address(
            ETHAddress::from_slice(eth_token_address.as_bytes()).map_err(|err| anyhow!(err))?,
        )
        .eth_contract_address(
            ETHAddress::from_slice(eth_contract_address.as_bytes()).map_err(|err| anyhow!(err))?,
        )
        .build();
    Ok(args)
}

// fn build_cell_script(args: ETHBridgeLockArgs, code_hash: &[u8]) -> Result<Script> {
//     let script = Script::new_builder()
//         .code_hash(Byte32::from_slice(&code_hash)?)
//         .hash_type(ScriptHashType::Data.into())
//         .args(args.as_bytes().pack())
//         .build();
//     Ok(script)
// }

pub fn deploy(
    rpc_client: &mut HttpRpcClient,
    indexer_client: &mut IndexerRpcClient,
    privkey: &SecretKey,
    data: Vec<Vec<u8>>,
    eth_dag_path: &str,
    // token_cell_script: Script,
    // eth_cell_script: Script,
) -> Result<TransactionView, String> {
    let from_pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let from_address_payload = AddressPayload::from_pubkey(&from_pubkey);
    let lockscript = Script::from(&from_address_payload);
    let mut tx_helper = TxHelper::default();
    for data in data.into_iter() {
        let output = CellOutput::new_builder()
            .lock((&from_address_payload).into())
            .build();
        tx_helper.add_output_with_auto_capacity(output, data.into());
    }
    // let output_token = CellOutput::new_builder()
    //     .capacity(Capacity::shannons(PUBLIC_BRIDGE_CELL).pack())
    //     .lock(token_cell_script)
    //     .build();
    // tx_helper.add_output_with_auto_capacity(output_token, ckb_types::bytes::Bytes::default());
    // let output_eth = CellOutput::new_builder()
    //     .capacity(Capacity::shannons(PUBLIC_BRIDGE_CELL).pack())
    //     .lock(eth_cell_script)
    //     .build();
    // tx_helper.add_output_with_auto_capacity(output_eth, ckb_types::bytes::Bytes::default());
    let output_dag = CellOutput::new_builder().lock(lockscript.clone()).build();
    let dep_data_raw = read_roots_collection_raw(eth_dag_path)?;
    let mut dag_root = vec![];
    for i in 0..dep_data_raw.dag_merkle_roots.len() {
        dag_root.push(hex::encode(&dep_data_raw.dag_merkle_roots[i].0).clone());
    }
    let dep_data_string = RootsCollectionJson {
        dag_merkle_roots: dag_root,
    };
    // dbg!(dep_data_string.clone());
    let dep_data: DagsMerkleRoots = dep_data_string.try_into().unwrap();
    // dbg!(hex::encode(dep_data.as_slice()));
    tx_helper.add_output_with_auto_capacity(output_dag, dep_data.as_bytes());
    let genesis_block: BlockView = rpc_client
        .get_block_by_number(0)?
        .expect("Can not get genesis block?")
        .into();
    let genesis_info = GenesisInfo::from_block(&genesis_block)?;
    let tx = tx_helper.supply_capacity(
        rpc_client,
        indexer_client,
        lockscript,
        &genesis_info,
        99_999_999,
    )?;
    sign(tx, rpc_client, privkey)
}
