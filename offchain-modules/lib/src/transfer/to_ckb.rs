use crate::util::ckb_util::{ETHSPVProofJson, Generator};
use crate::util::eth_util::{convert_eth_address, Web3Client};
use crate::util::settings::{OutpointConf, ScriptConf, Settings};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_sdk::{AddressPayload, GenesisInfo, HttpRpcClient, SECP256K1};
use ckb_types::core::{BlockView, DepType};
use ckb_types::packed::{Byte32, CellOutput, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ethabi::{Function, Param, ParamType, Token};
use force_eth_types::generated::basic::ETHAddress;
use force_eth_types::generated::eth_bridge_lock_cell::ETHBridgeLockArgs;
use force_sdk::indexer::IndexerRpcClient;
use force_sdk::tx_helper::{deploy, sign, TxHelper};
use force_sdk::util::{parse_privkey_path, send_tx_sync};
use log::info;
use serde_json::json;
use std::default::Default;
use web3::types::{H160, H256, U256};

pub async fn approve(
    from: H160,
    to: H160,
    url: String,
    key_path: String,
    wait: bool,
) -> Result<H256> {
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
    let tokens = [Token::Address(from), Token::Uint(U256::max_value())];
    let input_data = function.encode_input(&tokens)?;
    let res = rpc_client
        .send_transaction(to, key_path, input_data, U256::from(0), wait)
        .await?;
    Ok(res)
}

pub async fn lock_token(
    to: H160,
    url: String,
    key_path: String,
    data: &[Token],
    wait: bool,
) -> Result<H256> {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "lockToken".to_owned(),
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
                name: "bridgeFee".to_owned(),
                kind: ParamType::Uint(256),
            },
            Param {
                name: "recipientLockscript".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "replayResistOutpoint".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "sudtExtraData".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.encode_input(data)?;
    let res = rpc_client
        .send_transaction(to, key_path, input_data, U256::from(0), wait)
        .await?;
    Ok(res)
}

pub async fn lock_eth(
    to: H160,
    url: String,
    key_path: String,
    data: &[Token],
    eth_value: U256,
    wait: bool,
) -> Result<H256> {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "lockETH".to_owned(),
        inputs: vec![
            Param {
                name: "bridgeFee".to_owned(),
                kind: ParamType::Uint(256),
            },
            Param {
                name: "recipientLockscript".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "replayResistOutpoint".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "sudtExtraData".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.encode_input(data)?;
    let res = rpc_client
        .send_transaction(to, key_path, input_data, eth_value, wait)
        .await?;
    Ok(res)
}

pub async fn get_header_rlp(url: String, hash: H256) -> Result<String> {
    let mut rpc_client = Web3Client::new(url);
    Ok(rpc_client.get_header_rlp(hash.into()).await?)
}

pub async fn send_eth_spv_proof_tx(
    generator: &mut Generator,
    eth_proof: &ETHSPVProofJson,
    private_key_path: String,
    cell_dep: String,
) -> Result<ckb_types::H256> {
    let from_privkey = parse_privkey_path(private_key_path.as_str())?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);

    let unsigned_tx = generator.generate_eth_spv_tx(from_lockscript, eth_proof, cell_dep)?;
    let tx =
        sign(unsigned_tx, &mut generator.rpc_client, &from_privkey).map_err(|err| anyhow!(err))?;
    log::info!(
        "tx: \n{}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .map_err(|err| anyhow!(err))?
    );
    let tx_hash =
        send_tx_sync(&mut generator.rpc_client, &tx, 60).map_err(|e| anyhow::anyhow!(e))?;
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
pub fn dev_init(
    config_path: String,
    rpc_url: String,
    indexer_url: String,
    private_key_path: String,
    bridge_typescript_path: String,
    bridge_lockscript_path: String,
    light_client_typescript_path: String,
    recipient_typescript_path: String,
    sudt_path: String,
) -> Result<()> {
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let mut indexer_client = IndexerRpcClient::new(indexer_url);
    let private_key = parse_privkey_path(&private_key_path)?;

    // dev deploy
    let bridge_typescript_bin = std::fs::read(bridge_typescript_path)?;
    let bridge_lockscript_bin = std::fs::read(bridge_lockscript_path)?;
    let light_client_typescript_bin = std::fs::read(light_client_typescript_path)?;
    let recipient_typescript_bin = std::fs::read(recipient_typescript_path)?;
    let sudt_bin = std::fs::read(sudt_path)?;

    let bridge_typescript_code_hash = blake2b_256(&bridge_typescript_bin);
    let bridge_typescript_code_hash_hex = hex::encode(&bridge_typescript_code_hash);

    let light_client_typescript_code_hash = blake2b_256(&light_client_typescript_bin);
    let light_client_typescript_code_hash_hex = hex::encode(&light_client_typescript_code_hash);

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
        recipient_typescript_bin,
        sudt_bin,
    ];

    let tx = deploy(&mut rpc_client, &mut indexer_client, &private_key, data)
        .map_err(|err| anyhow!(err))?;
    let tx_hash = send_tx_sync(&mut rpc_client, &tx, 60).map_err(|err| anyhow!(err))?;
    let tx_hash_hex = hex::encode(tx_hash.as_bytes());

    let settings = Settings {
        bridge_lockscript: ScriptConf {
            code_hash: bridge_lockscript_code_hash_hex.clone(),
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 0,
            },
        },
        bridge_typescript: ScriptConf {
            code_hash: bridge_typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 1,
            },
        },
        light_client_typescript: ScriptConf {
            code_hash: light_client_typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 2,
            },
        },
        recipient_typescript: ScriptConf {
            code_hash: recipient_typescript_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 3,
            },
        },
        sudt: ScriptConf {
            code_hash: sudt_code_hash_hex,
            outpoint: OutpointConf {
                tx_hash: tx_hash_hex.clone(),
                index: 4,
            },
        },
    };
    log::info!("settings: {:?}", &settings);
    settings.write(&config_path).map_err(|e| anyhow!(e))?;
    println!("force-bridge-eth config written to {}", &config_path);
    Ok(())
}

pub fn create_bridge_cell(
    config_path: String,
    rpc_url: String,
    indexer_url: String,
    private_key_path: String,
    eth_contract_address_str: String,
    eth_token_address_str: String,
    recipient_address: String,
) -> Result<()> {
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let mut indexer_client = IndexerRpcClient::new(indexer_url);
    let privkey = parse_privkey_path(&private_key_path)?;
    let from_pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let from_address_payload = AddressPayload::from_pubkey(&from_pubkey);
    let from_lockscript = Script::from(&from_address_payload);
    let settings = Settings::new(&config_path)?;
    let mut tx_helper = TxHelper::default();

    // build bridge lockscrpt
    let eth_contract_address = convert_eth_address(eth_contract_address_str.as_str())?;
    let eth_token_address = convert_eth_address(eth_token_address_str.as_str())?;
    let token_args = build_eth_bridge_lock_args(eth_token_address, eth_contract_address)?;
    let token_cell_lockscript = build_cell_script(
        token_args,
        &hex::decode(settings.bridge_lockscript.code_hash)?,
    )?;

    // build bridge typescript
    // todo

    let output = CellOutput::new_builder()
        .lock(token_cell_lockscript)
        .build();
    tx_helper.add_output_with_auto_capacity(output, ckb_types::bytes::Bytes::default());
    let genesis_block: BlockView = rpc_client
        .get_block_by_number(0)
        .expect("genesis not exists")
        .expect("Can not get genesis block?")
        .into();
    let genesis_info = GenesisInfo::from_block(&genesis_block).map_err(|e| anyhow!(e))?;
    let raw_tx = tx_helper
        .supply_capacity(
            &mut rpc_client,
            &mut indexer_client,
            from_lockscript,
            &genesis_info,
            99_999_999,
        )
        .map_err(|e| anyhow!(e))?;
    let tx = sign(raw_tx, &mut rpc_client, &privkey).map_err(|e| anyhow!("sign error {}", e))?;
    let tx_hash = send_tx_sync(&mut rpc_client, &tx, 60).map_err(|err| anyhow!(err))?;
    let outpoint = OutPoint::new_builder()
        .tx_hash(Byte32::from_slice(tx_hash.as_ref())?)
        .build();
    let outpoint_hex = hex::encode(outpoint.as_slice());
    info!(
        "create bridge cell successfully for {}, outpoint: {}",
        recipient_address, &outpoint_hex
    );
    println!("{}", json!({ "outpoint": outpoint_hex }));
    Ok(())
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

fn build_cell_script(args: ETHBridgeLockArgs, code_hash: &[u8]) -> Result<Script> {
    let script = Script::new_builder()
        .code_hash(Byte32::from_slice(&code_hash)?)
        .hash_type(DepType::Code.into())
        .args(args.as_bytes().pack())
        .build();
    Ok(script)
}
