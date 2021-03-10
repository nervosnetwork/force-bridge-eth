use super::errors::RpcError;
use super::types::*;
use super::REPLAY_RESIST_CELL_NUMBER;
use super::{DappState, ReplayResistTask};
use crate::dapp::db::server::{self as db, add_replay_resist_cells, is_token_replay_resist_init};
use crate::util::ckb_util::{
    build_lockscript_from_address, get_sudt_type_script, parse_cell, parse_merkle_cell_data,
};
use crate::util::eth_util::{
    build_lock_eth_payload, build_lock_token_payload, convert_eth_address, make_transaction,
    rlp_transaction, Web3Client,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use ckb_jsonrpc_types::{Script as ScriptJson, Uint128, Uint64};
use ckb_sdk::{Address, HumanCapacity};
use ckb_types::packed::{Script, ScriptReader};
use ethabi::Token;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use molecule::prelude::{Entity, Reader};
use serde_json::{json, Value};
use std::convert::TryFrom;
use std::str::FromStr;
use tokio::sync::oneshot;
use web3::types::{CallRequest, U256};

#[post("/init_token")]
pub async fn init_token(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: InitTokenArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("init token args: {:?}", args);
    if args.token_address.len() != 40 {
        return Err(RpcError::BadRequest(
            "invalid args: token address string length should be 40".to_string(),
        ));
    }
    let _ = data.init_token_mutex.try_lock().map_err(|_| {
        RpcError::BadRequest("init_token api should be serial accessed".to_string())
    })?;
    let is_token_init = is_token_replay_resist_init(&data.db, args.token_address.as_str())
        .await
        .map_err(|e| {
            RpcError::ServerError(format!("get is_token_replay_resist_init error: {}", e))
        })?;
    if is_token_init {
        return Err(RpcError::BadRequest("token already inited".to_string()));
    }
    let cells = data
        .get_or_create_bridge_cell(
            args.token_address.as_str(),
            REPLAY_RESIST_CELL_NUMBER,
            data.init_token_privkey.clone(),
            false,
        )
        .await
        .map_err(|e| RpcError::ServerError(format!("get or create bridge cell error: {}", e)))?;
    add_replay_resist_cells(&data.db, &cells, args.token_address.as_str())
        .await
        .map_err(|e| {
            RpcError::ServerError(format!("add replay resist cells to db error: {}", e))
        })?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/lock")]
pub async fn lock(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: LockArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("lock args: {:?}", args);
    let lock_sender = convert_eth_address(args.sender.as_str())
        .map_err(|e| RpcError::BadRequest(format!("sender address parse fail: {}", e)))?;
    let to = convert_eth_address(data.deployed_contracts.eth_token_locker_addr.as_str())
        .map_err(|e| RpcError::BadRequest(format!("lock contract address parse fail: {}", e)))?;

    let nonce = U256::from(u128::from(args.nonce));
    let gas_price = U256::from(u128::from(args.gas_price));
    let amount = U256::from(u128::from(args.amount));
    let bridge_fee = U256::from(u128::from(args.bridge_fee));
    if args.token_address.len() != 40 {
        return Err(RpcError::BadRequest(
            "invalid args: token address string length should be 40".to_string(),
        ));
    }
    let is_token_init = is_token_replay_resist_init(&data.db, args.token_address.as_str())
        .await
        .map_err(|e| {
            RpcError::ServerError(format!("get is_token_replay_resist_init error: {}", e))
        })?;
    if !is_token_init {
        return Err(RpcError::BadRequest("token not init".to_string()));
    }
    let token_addr = convert_eth_address(&args.token_address)
        .map_err(|e| RpcError::BadRequest(format!("token address parse fail: {}", e)))?;

    let recipient_lockscript = build_lockscript_from_address(&args.ckb_recipient_address)
        .map_err(|e| RpcError::BadRequest(format!("ckb recipient address parse fail: {}", e)))?;

    let web3_client = data.get_web3_client().client().clone();

    let (sender, receiver) = oneshot::channel();
    let replay_resist_task = ReplayResistTask {
        token: args.token_address.clone(),
        resp: sender,
    };
    data.replay_resist_sender
        .clone()
        .try_send(replay_resist_task)?;
    let replay_resist_outpoint = receiver
        .await
        .map_err(|e| {
            RpcError::ServerError(format!("receive replay resist cell channel error: {}", e))
        })?
        .map_err(|e| {
            RpcError::ServerError(format!("receive replay resist cell result error: {}", e))
        })?;

    let data = [
        Token::Address(token_addr),
        Token::Uint(amount),
        Token::Uint(bridge_fee),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(hex::decode(&replay_resist_outpoint).map_err(|e| {
            RpcError::ServerError(format!("decode replay_resist_outpoint fail, err: {}", e))
        })?),
        Token::Bytes(
            hex::decode(&args.sudt_extra_data)
                .map_err(|e| RpcError::BadRequest(format!("decode sudt_extra_data fail: {}", e)))?,
        ),
    ];
    let mut eth_value = amount;
    let input_data = {
        if token_addr.0 == [0u8; 20] {
            let lock_eth_data = &data[2..];
            build_lock_eth_payload(lock_eth_data).map_err(|e| {
                RpcError::ServerError(format!("abi encode lock eth data fail, err: {}", e))
            })?
        } else {
            eth_value = U256::from(0);
            build_lock_token_payload(&data).map_err(|e| {
                RpcError::ServerError(format!("abi encode lock token data fail, err: {}", e))
            })?
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
        .map_err(|e| RpcError::ServerError(format!("estimate gas failed: {:?}", e)))?;

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
    let args: BurnArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("burn args: {:?}", args);

    let from_lockscript = Script::from(
        Address::from_str(args.from_lockscript_addr.as_str())
            .map_err(|e| RpcError::BadRequest(format!("ckb_address to script fail: {}", e)))?
            .payload(),
    );
    let token_address = convert_eth_address(args.token_address.as_str())
        .map_err(|e| RpcError::BadRequest(format!("convert token address error: {}", e)))?;
    let lock_contract_address = convert_eth_address(
        data.deployed_contracts.eth_token_locker_addr.as_str(),
    )
    .map_err(|e| RpcError::BadRequest(format!("convert lock contract address error: {}", e)))?;
    let recipient_address = convert_eth_address(args.recipient_address.as_str())
        .map_err(|e| RpcError::BadRequest(format!("convert recipient address error: {}", e)))?;

    let mut generator = data
        .get_generator()
        .await
        .map_err(|e| RpcError::ServerError(format!("get_generator: {:?}", e)))?;
    let tx_fee: u64 =
        HumanCapacity::from_str(&args.tx_fee.clone().unwrap_or_else(|| "0.0001".to_string()))
            .map_err(|e| RpcError::BadRequest(format!("tx fee invalid: {}", e)))?
            .into();
    let tx = generator
        .burn(
            tx_fee,
            from_lockscript,
            args.unlock_fee.into(),
            args.amount.into(),
            token_address,
            lock_contract_address,
            recipient_address,
        )
        .map_err(|e| RpcError::ServerError(format!("generate burn tx error: {}", e)))?;
    let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    Ok(HttpResponse::Ok().json(BurnResult { raw_tx: rpc_tx }))
}

#[post("/get_eth_to_ckb_status")]
pub async fn get_eth_to_ckb_status(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetEthToCkbStatusArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("get_eth_to_ckb_status args: {:?}", args);

    if args.eth_lock_tx_hash.len() != 64 {
        return Err(RpcError::BadRequest(
            "invalid args: lock tx hash string length should be 64".to_string(),
        ));
    }
    let indexer_status = db::get_eth_to_ckb_indexer_status(&data.db, &args.eth_lock_tx_hash)
        .await
        .map_err(|e| RpcError::ServerError(format!("get_eth_to_ckb_indexer_status: {:?}", e)))?
        .ok_or_else(|| {
            RpcError::ServerError(format!("eth lock tx {} not found", &args.eth_lock_tx_hash))
        })?;
    let mut res = GetEthToCkbStatusResponse {
        eth_lock_tx_hash: indexer_status.eth_lock_tx_hash,
        status: indexer_status.status.clone(),
        err_msg: "".to_string(),
        token_addr: indexer_status.token_addr,
        sender_addr: indexer_status.sender_addr,
        locked_amount: indexer_status.locked_amount,
        bridge_fee: indexer_status.bridge_fee,
        ckb_recipient_lockscript: indexer_status.ckb_recipient_lockscript,
        sudt_extra_data: indexer_status.sudt_extra_data,
        ckb_tx_hash: indexer_status.ckb_tx_hash,
        block_number: indexer_status.eth_block_number,
        replay_resist_outpoint: indexer_status.replay_resist_outpoint,
    };
    if indexer_status.status == "success" {
        return Ok(HttpResponse::Ok().json(res));
    }
    let relay_status_opt = db::get_eth_to_ckb_relay_status(&data.db, &args.eth_lock_tx_hash)
        .await
        .map_err(|e| RpcError::ServerError(format!("get_eth_to_ckb_relay_status: {:?}", e)))?;
    if relay_status_opt.is_none() || relay_status_opt.clone().unwrap().status == "retryable" {
        return Ok(HttpResponse::Ok().json(res));
    }
    let relay_status = relay_status_opt.unwrap();
    res.status = relay_status.status;
    res.err_msg = relay_status.err_msg;
    Ok(HttpResponse::Ok().json(res))
}

#[post("/get_ckb_to_eth_status")]
pub async fn get_ckb_to_eth_status(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetCkbToEthStatusArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("get_ckb_to_eth_status args: {:?}", args);

    let status = db::get_ckb_to_eth_status(&data.db, &args.ckb_burn_tx_hash)
        .await
        .map_err(|e| RpcError::ServerError(format!("get_ckb_to_eth_status: {:?}", e)))?
        .ok_or_else(|| {
            RpcError::ServerError(format!("ckb burn tx {} not found", &args.ckb_burn_tx_hash))
        })?;
    Ok(HttpResponse::Ok().json(status))
}

#[post("/get_crosschain_history")]
pub async fn get_crosschain_history(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetCrosschainHistoryArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("get_crosschain_history args: {:?}", args);
    let mut crosschain_history = GetCrosschainHistoryRes::default();
    // eth to ckb history
    if let Some(lock_sender_addr) = args.lock_sender_addr {
        if lock_sender_addr.len() != 40 {
            return Err(RpcError::BadRequest(
                "invalid args: lock_sender_addr string length should be 40".to_string(),
            ));
        }
        let raw_crosschain_history =
            db::get_eth_to_ckb_crosschain_history(&data.db, &lock_sender_addr)
                .await
                .map_err(|e| {
                    RpcError::ServerError(format!(
                        "get_eth_to_ckb_crosschain_history error: {:?}",
                        e
                    ))
                })?;
        let res: Result<Vec<_>, _> = raw_crosschain_history
            .into_iter()
            .map(EthToCkbCrosschainHistoryRes::try_from)
            .collect();
        crosschain_history.eth_to_ckb = res?;
    }
    // ckb to eth
    if let Some(eth_recipient_addr) = args.eth_recipient_addr {
        if eth_recipient_addr.len() != 40 {
            return Err(RpcError::BadRequest(
                "invalid args: eth_recipient_addr string length should be 40".to_string(),
            ));
        }
        let raw_crosschain_history =
            db::get_ckb_to_eth_crosschain_history(&data.db, &eth_recipient_addr)
                .await
                .map_err(|e| {
                    RpcError::ServerError(format!(
                        "get_ckb_to_eth_crosschain_history error: {:?}",
                        e
                    ))
                })?;
        crosschain_history.ckb_to_eth = raw_crosschain_history
            .into_iter()
            .map(CkbToEthCrosschainHistoryRes::from)
            .collect();
    }
    Ok(HttpResponse::Ok().json(crosschain_history))
}

#[post("/get_sudt_balance")]
pub async fn get_sudt_balance(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetSudtBalanceArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    log::info!("get_sudt_balance args: {:?}", args);

    let token_address = convert_eth_address(args.token_address.as_str())
        .map_err(|e| RpcError::BadRequest(format!("token address parse fail: {}", e)))?;
    let lock_contract_address = convert_eth_address(
        data.deployed_contracts.eth_token_locker_addr.as_str(),
    )
    .map_err(|e| RpcError::BadRequest(format!("lock contract address parse fail: {}", e)))?;
    let sudt_script: ScriptJson = get_sudt_type_script(
        &data.deployed_contracts,
        token_address,
        lock_contract_address,
    )
    .map_err(|e| RpcError::ServerError(format!("get_sudt_type_script: {}", e)))?
    .into();

    let mut generator = data
        .get_generator()
        .await
        .map_err(|e| RpcError::ServerError(format!("get_generator: {}", e)))?;
    let addr_lockscript: Script = {
        if args.address.is_some() {
            Address::from_str(&args.address.unwrap())
                .map_err(|e| RpcError::BadRequest(format!("ckb address error: {}", e)))?
                .payload()
                .into()
        } else if args.script.is_some() {
            let script = hex::decode(args.script.unwrap())
                .map_err(|e| RpcError::BadRequest(format!("invalid ckb_script: {}", e)))?;
            ScriptReader::verify(&script, false)
                .map_err(|e| RpcError::BadRequest(format!("invalid ckb_script: {}", e)))?;
            Script::from_slice(&script)
                .map_err(|e| RpcError::BadRequest(format!("invalid ckb_script: {}", e)))?
        } else {
            return Err(RpcError::BadRequest(
                "ckb_address or ckb_script should be provided".to_string(),
            ));
        }
    };
    let balance: Uint128 = generator
        .get_sudt_balance(addr_lockscript, token_address, lock_contract_address)
        .map_err(|e| RpcError::ServerError(format!("get_sudt_balance: {}", e)))?
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
    let args: GetBestBlockHeightArgs = serde_json::from_value(args.into_inner())
        .map_err(|e| RpcError::BadRequest(format!("invalid args: {}", e)))?;
    match args.chain.as_str() {
        "ckb" => {
            let contract_address = convert_eth_address(&data.deployed_contracts.eth_ckb_chain_addr)
                .map_err(|e| RpcError::ServerError(format!("eth_ckb_chain_addr invalid: {}", e)))?;
            let mut eth_client = Web3Client::new(data.eth_rpc_url.clone());
            let result = eth_client
                .get_contract_height("latestBlockNumber", contract_address)
                .await
                .map_err(|e| {
                    RpcError::ServerError(format!(
                        "eth client call get_contract_height, err: {}",
                        e
                    ))
                })?;
            Ok(HttpResponse::Ok().json(Uint64::from(result)))
        }
        "eth" => {
            let mut generator = data
                .get_generator()
                .await
                .map_err(|e| RpcError::ServerError(format!("get_generator: {}", e)))?;
            let script = parse_cell(
                data.deployed_contracts
                    .light_client_cell_script
                    .cell_script
                    .as_str(),
            )
            .map_err(|e| {
                RpcError::ServerError(format!("get light client typescript fail: {:?}", e))
            })?;
            let cell = get_live_cell_by_typescript(&mut generator.indexer_client, script)
                .map_err(|e| RpcError::ServerError(format!("get live cell fail: {:?}", e)))?
                .ok_or_else(|| RpcError::ServerError("eth client cell not exist".to_string()))?;
            let ckb_cell_data = cell.output_data.as_bytes().to_vec();
            let (_, latest_height, _) = parse_merkle_cell_data(ckb_cell_data).map_err(|e| {
                RpcError::ServerError(format!("parse merkle cell data fail: {:?}", e))
            })?;
            Ok(HttpResponse::Ok().json(Uint64::from(latest_height)))
        }
        _ => {
            return Err(RpcError::BadRequest(
                "unknown chain type, only support eth and ckb".to_string(),
            ));
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
