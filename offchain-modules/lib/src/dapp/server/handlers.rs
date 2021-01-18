use super::error::RpcError;
use super::types::*;
use super::{DappState, ReplayResistTask};
use crate::dapp::db::server::{self as db, CrosschainHistory};
use crate::util::ckb_util::{
    build_lockscript_from_address, get_sudt_type_script, parse_cell, parse_main_chain_headers,
};
use crate::util::eth_util::{
    build_lock_eth_payload, build_lock_token_payload, convert_eth_address, make_transaction,
    rlp_transaction, Web3Client,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::anyhow;
use ckb_jsonrpc_types::{Script as ScriptJson, Uint128, Uint64};
use ckb_sdk::{Address, HumanCapacity};
use ckb_types::packed::{Script, ScriptReader};
use ethabi::Token;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use molecule::prelude::{Entity, Reader, ToOwned};
use serde_json::{json, Value};
use std::str::FromStr;
use tokio::sync::oneshot;
use web3::types::{CallRequest, U256};

#[post("/lock")]
pub async fn lock(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: LockArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    log::info!("lock args: {:?}", args);
    let lock_sender = convert_eth_address(args.sender.as_str())
        .map_err(|e| format!("sender address parse fail: {}", e))?;
    let to = convert_eth_address(data.deployed_contracts.eth_token_locker_addr.as_str())
        .map_err(|e| format!("lock contract address parse fail: {}", e))?;
    let nonce = U256::from(u128::from(args.nonce));
    let gas_price = U256::from(u128::from(args.gas_price));
    let amount = U256::from(u128::from(args.amount));
    let bridge_fee = U256::from(u128::from(args.bridge_fee));
    let token_addr = convert_eth_address(&args.token_address)
        .map_err(|e| format!("token address parse fail: {}", e))?;
    let recipient_lockscript = build_lockscript_from_address(&args.ckb_recipient_address)
        .map_err(|e| format!("ckb recipient address parse fail: {}", e))?;
    let web3_client = data.get_web3_client().client().clone();

    let (sender, receiver) = oneshot::channel();
    let replay_resist_task = ReplayResistTask {
        token: args.token_address.clone(),
        resp: sender,
    };
    let _ = data
        .replay_resist_sender
        .clone()
        .send(replay_resist_task)
        .await;
    let replay_resist_outpoint = receiver.await.unwrap()?;

    let data = [
        Token::Address(token_addr),
        Token::Uint(amount),
        Token::Uint(bridge_fee),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(
            hex::decode(&replay_resist_outpoint)
                .map_err(|e| format!("decode replay_resist_outpoint fail, err: {}", e))?,
        ),
        Token::Bytes(
            hex::decode(&args.sudt_extra_data)
                .map_err(|e| format!("decode sudt_extra_data fail, err: {}", e))?,
        ),
    ];
    let mut eth_value = amount;
    let input_data = {
        if token_addr.0 == [0u8; 20] {
            let lock_eth_data = &data[2..];
            build_lock_eth_payload(lock_eth_data)
                .map_err(|e| format!("abi encode lock eth data fail, err: {}", e))?
        } else {
            eth_value = U256::from(0);
            build_lock_token_payload(&data)
                .map_err(|e| format!("abi encode lock token data fail, err: {}", e))?
        }
    };
    let gas_limit = web3_client
        .eth()
        .estimate_gas(
            CallRequest {
                from: Some(lock_sender),
                to: Some(to),
                gas: None,
                gas_price: None,
                value: Some(eth_value),
                data: Some(input_data.clone().into()),
            },
            None,
        )
        .await
        .map_err(|e| format!("estimate gas failed: {:?}", e))?;

    let raw_transaction = make_transaction(to, nonce, input_data, gas_price, gas_limit, eth_value);
    let result = LockResult {
        nonce: raw_transaction.nonce,
        to: raw_transaction.to,
        value: raw_transaction.value,
        gas_price: raw_transaction.gas_price,
        gas: raw_transaction.gas,
        data: hex::encode(raw_transaction.clone().data),
        raw: rlp_transaction(&raw_transaction),
    };
    Ok(HttpResponse::Ok().json(result))
}

#[post("/burn")]
pub async fn burn(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: BurnArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    let from_lockscript = Script::from(
        Address::from_str(args.from_lockscript_addr.as_str())
            .map_err(|err| format!("ckb_address to script fail: {}", err))?
            .payload(),
    );
    let token_address = convert_eth_address(args.token_address.as_str())?;
    let lock_contract_address =
        convert_eth_address(data.deployed_contracts.eth_token_locker_addr.as_str())?;
    let recipient_address = convert_eth_address(args.recipient_address.as_str())?;

    let mut generator = data.get_generator().await?;
    let tx_fee: u64 =
        HumanCapacity::from_str(&args.tx_fee.clone().unwrap_or_else(|| "0.0001".to_string()))?
            .into();
    let tx = generator.burn(
        tx_fee,
        from_lockscript,
        args.unlock_fee.into(),
        args.amount.into(),
        token_address,
        lock_contract_address,
        recipient_address,
    )?;
    let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    log::info!(
        "burn args: {} tx: {}",
        serde_json::to_string_pretty(&args).unwrap(),
        serde_json::to_string_pretty(&rpc_tx).unwrap()
    );
    Ok(HttpResponse::Ok().json(BurnResult { raw_tx: rpc_tx }))
}

#[post("/get_eth_to_ckb_status")]
pub async fn get_eth_to_ckb_status(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: EthLockTxHash =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    let status = db::get_eth_to_ckb_status(&data.db, &args.eth_lock_tx_hash)
        .await?
        .ok_or(format!("eth lock tx {} not found", &args.eth_lock_tx_hash))?;
    Ok(HttpResponse::Ok().json(status))
}

#[post("/get_ckb_to_eth_status")]
pub async fn get_ckb_to_eth_status(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: CkbBurnTxHash =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    let status = db::get_ckb_to_eth_status(&data.db, &args.ckb_burn_tx_hash)
        .await?
        .ok_or(format!("ckb burn tx {} not found", &args.ckb_burn_tx_hash))?;
    Ok(HttpResponse::Ok().json(status))
}

#[post("/get_crosschain_history")]
pub async fn get_crosschain_history(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetCrosschainHistoryArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    log::debug!("get_crosschain_history args: {:?}", args);
    let mut crosschain_history = GetCrosschainHistoryRes::default();
    // eth to ckb history
    let mut ckb_recipient_lockscript = None;
    if let Some(lockscript_raw) = args.ckb_recipient_lockscript {
        ckb_recipient_lockscript = Some(lockscript_raw);
    }
    if let Some(addr) = args.ckb_recipient_lockscript_addr {
        let from_lockscript = Script::from(
            Address::from_str(&addr)
                .map_err(|err| format!("ckb_address to script fail: {}", err))?
                .payload(),
        );
        ckb_recipient_lockscript = Some(hex::encode(from_lockscript.as_slice()))
    }
    log::debug!(
        "ckb_recipient_lockscript args: {:?}",
        ckb_recipient_lockscript
    );
    if let Some(lockscript) = ckb_recipient_lockscript {
        crosschain_history.eth_to_ckb =
            db::get_eth_to_ckb_crosschain_history(&data.db, &lockscript).await?;
    }
    // ckb to eth
    if let Some(eth_recipient_addr) = args.eth_recipient_addr {
        crosschain_history.ckb_to_eth =
            db::get_ckb_to_eth_crosschain_history(&data.db, &eth_recipient_addr).await?;
    }
    Ok(HttpResponse::Ok().json(format_crosschain_history_res(&crosschain_history)))
}

#[post("/get_sudt_balance")]
pub async fn get_sudt_balance(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetSudtBalanceArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    let token_address = convert_eth_address(args.token_address.as_str())
        .map_err(|e| format!("token address parse fail: {}", e))?;
    let lock_contract_address =
        convert_eth_address(data.deployed_contracts.eth_token_locker_addr.as_str())
            .map_err(|e| format!("lock contract address parse fail: {}", e))?;
    let sudt_script: ScriptJson = get_sudt_type_script(
        &data.deployed_contracts,
        token_address,
        lock_contract_address,
    )?
    .into();

    let mut generator = data.get_generator().await?;

    let addr_lockscript: Script = {
        if args.address.is_some() {
            Address::from_str(&args.address.unwrap())
                .map_err(|err| anyhow!(err))?
                .payload()
                .into()
        } else if args.script.is_some() {
            let script = hex::decode(args.script.unwrap())
                .map_err(|e| anyhow!("invalid ckb_script, err: {}", e))?;
            ScriptReader::verify(&script, false)
                .map_err(|e| anyhow!("invalid ckb_script, err: {}", e))?;
            Script::from_slice(&script).map_err(|e| anyhow!("invalid ckb_script, err: {}", e))?
        } else {
            return Err(anyhow!("ckb_address or ckb_script should be provided").into());
        }
    };
    let balance: Uint128 = generator
        .get_sudt_balance(addr_lockscript, token_address, lock_contract_address)
        .map_err(|e| format!("get_sudt_balance fail, err: {}", e))?
        .into();
    Ok(HttpResponse::Ok().json(json! ({
        "balance": balance,
        "sudt_script": sudt_script,
    })))
}

#[post("/get_best_block_height")]
pub async fn get_best_block_height(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetBestBlockHeightArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    match args.chain.as_str() {
        "ckb" => {
            let contract_address = convert_eth_address(&data.deployed_contracts.eth_ckb_chain_addr)
                .map_err(|e| format!("abi encode lock eth data fail, err: {}", e))?;

            let mut eth_client = Web3Client::new(data.eth_rpc_url.clone());

            let result = eth_client
                .get_contract_height("latestBlockNumber", contract_address)
                .await
                .map_err(|e| format!("eth client call get_contract_height, err: {}", e))?;
            Ok(HttpResponse::Ok().json(Uint64::from(result)))
        }
        "eth" => {
            let mut generator = data.get_generator().await?;

            let script = parse_cell(
                data.deployed_contracts
                    .light_client_cell_script
                    .cell_script
                    .as_str(),
            )
            .map_err(|e| format!("get typescript fail {:?}", e))?;

            let cell = get_live_cell_by_typescript(&mut generator.indexer_client, script)
                .map_err(|e| format!("get live cell fail: {}", e))?
                .ok_or("eth header cell not exist")?;

            let (un_confirmed_headers, _) =
                parse_main_chain_headers(cell.output_data.as_bytes().to_vec())
                    .map_err(|e| format!("parse header data fail: {}", e))?;

            let best_header = un_confirmed_headers.last().ok_or("header is none")?;
            let best_block_number = best_header.number.ok_or("header number is none")?;
            Ok(HttpResponse::Ok().json(Uint64::from(best_block_number.as_u64())))
        }
        _ => {
            return Err("unknown chain type, only support eth and ckb"
                .to_string()
                .into())
        }
    }
}

#[get("/")]
pub async fn index() -> impl Responder {
    "Nervos force bridge dapp server API endpoint"
}

#[get("/settings")]
pub async fn settings(data: web::Data<DappState>) -> impl Responder {
    HttpResponse::Ok().json(&data.deployed_contracts)
}

pub fn pad_0x_prefix(s: &str) -> String {
    if !s.starts_with("0x") && !s.starts_with("0X") {
        let mut res = "0x".to_owned();
        res.push_str(s);
        res
    } else {
        s.to_owned()
    }
}

pub fn format_crosschain_history(c: &CrosschainHistory) -> CrosschainHistory {
    let mut res = c.to_owned();
    res.eth_tx_hash = c.eth_tx_hash.as_ref().map(|h| pad_0x_prefix(&h));
    res.ckb_tx_hash = c.ckb_tx_hash.as_ref().map(|h| pad_0x_prefix(&h));
    res.token_addr = pad_0x_prefix(&c.token_addr);
    res
}

pub fn format_crosschain_history_res(res: &GetCrosschainHistoryRes) -> GetCrosschainHistoryRes {
    GetCrosschainHistoryRes {
        eth_to_ckb: res
            .eth_to_ckb
            .iter()
            .map(format_crosschain_history)
            .collect(),
        ckb_to_eth: res
            .ckb_to_eth
            .iter()
            .map(format_crosschain_history)
            .collect(),
    }
}
