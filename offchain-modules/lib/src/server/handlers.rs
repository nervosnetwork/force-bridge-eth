use super::error::RpcError;
use super::state::DappState;
use super::types::*;
use crate::server::proof_relayer::db::{update_eth_to_ckb_status, EthToCkbRecord};
use crate::server::proof_relayer::{db, handler};
use crate::transfer::to_ckb::create_bridge_cell;
use crate::util::ckb_util::{
    build_lockscript_from_address, get_sudt_type_script, parse_cell, parse_main_chain_headers,
};
use crate::util::eth_util::{
    build_lock_eth_payload, build_lock_token_payload, convert_eth_address, convert_hex_to_h256,
    make_transaction, rlp_transaction, Web3Client,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::anyhow;
use ckb_jsonrpc_types::{Script as ScriptJson, Uint128, Uint64};
use ckb_sdk::{Address, HumanCapacity};
use ckb_types::packed::{Script, ScriptReader};
use ethabi::Token;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use molecule::prelude::{Entity, Reader};
use serde_json::{json, Value};
use std::str::FromStr;
use web3::types::U256;

#[post("/get_or_create_bridge_cell")]
pub async fn get_or_create_bridge_cell(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: CreateBridgeCellArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    log::info!("get_or_create_bridge_cell args: {:?}", args);
    let tx_fee = "0.1".to_string();
    let capacity = "283".to_string();
    let outpoints = create_bridge_cell(
        data.config_path.clone(),
        data.network.clone(),
        data.ckb_private_key_path.clone(),
        tx_fee,
        capacity,
        args.eth_token_address.clone(),
        args.recipient_address.clone(),
        args.bridge_fee.into(),
        5,
    )
    .await?;
    Ok(HttpResponse::Ok().json(CreateBridgeCellResponse { outpoints }))
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

#[post("/get_crosschain_history")]
pub async fn get_crosschain_history(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: GetCrosschainHistoryArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    log::info!("get_crosschain_history args: {:?}", args);
    let ckb_recipient_lockscript = match args.ckb_recipient_lockscript {
        Some(lockscript_raw) => lockscript_raw,
        None => {
            let from_lockscript = Script::from(
                Address::from_str(
                    &args
                        .ckb_recipient_lockscript_addr
                        .ok_or_else(|| anyhow!("arg ckb_recipient_lockscript not provided"))?,
                )
                .map_err(|err| format!("ckb_address to script fail: {}", err))?
                .payload(),
            );
            hex::encode(from_lockscript.as_slice())
        }
    };
    log::info!(
        "ckb_recipient_lockscript args: {:?}",
        ckb_recipient_lockscript
    );
    let crosschain_history =
        db::get_crosschain_history(&data.db, &ckb_recipient_lockscript).await?;
    Ok(HttpResponse::Ok().json(json!({
        "crosschain_history": crosschain_history,
    })))
}

#[post("/relay_eth_to_ckb_proof")]
pub async fn relay_eth_to_ckb_proof(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: EthLockTxHash =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    log::info!("relay_eth_to_ckb_proof args: {:?}", args);
    let _eth_lock_tx_hash = convert_hex_to_h256(&args.eth_lock_tx_hash)
        .map_err(|e| format!("invalid tx hash {}. err: {}", &args.eth_lock_tx_hash, e))?;
    let eth_lock_tx_hash = args.eth_lock_tx_hash.clone();
    let status = db::get_eth_to_ckb_status(&data.db, &eth_lock_tx_hash).await?;
    if status.is_some() {
        return Ok(HttpResponse::Ok().json(json!({
            "message": "tx proof relay processing/processed"
        })));
    }
    let row_id = db::create_eth_to_ckb_status_record(&data.db, eth_lock_tx_hash.clone()).await?;
    let generator = data.get_generator().await?;
    tokio::spawn(async move {
        let mut record = EthToCkbRecord {
            id: row_id,
            eth_lock_tx_hash: eth_lock_tx_hash.clone(),
            status: "pending".to_string(),
            ..Default::default()
        };
        let res = handler::relay_eth_to_ckb_proof(
            record.clone(),
            data.eth_rpc_url.clone(),
            data.deployed_contracts.eth_token_locker_addr.clone(),
            generator,
            data.config_path.clone(),
            data.from_privkey,
            &data.db,
        )
        .await;
        match res {
            Ok(_) => {
                log::info!("relay eth_lock_tx_hash {} successfully", &eth_lock_tx_hash);
            }
            Err(e) => {
                log::error!(
                    "relay eth_lock_tx_hash {} failed, err: {}",
                    &eth_lock_tx_hash,
                    e
                );
                record.err_msg = Some(e.to_string());
                record.status = "error".to_string();
                let res = update_eth_to_ckb_status(&data.db, &record).await;
                if res.is_err() {
                    log::error!(
                        "save error msg for record {:?} failed, err: {}",
                        record,
                        res.unwrap_err()
                    )
                }
            }
        }
    });
    Ok(HttpResponse::Ok().json(json!({
        "message": "tx proof relay submitted"
    })))
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
    let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx.clone());
    log::info!(
        "burn args: {} tx: {}",
        serde_json::to_string_pretty(&args).unwrap(),
        serde_json::to_string_pretty(&rpc_tx).unwrap()
    );
    tokio::spawn(async move {
        for i in 0u8..10 {
            let res = handler::relay_ckb_to_eth_proof(
                data.config_path.clone(),
                data.eth_private_key_path.clone(),
                data.network.clone(),
                tx.clone(),
            )
            .await;
            match res {
                Ok(_) => break,
                Err(e) => {
                    log::error!("unlock failed. index: {}, err: {}", i, e);
                    tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
                }
            }
        }
    });
    Ok(HttpResponse::Ok().json(BurnResult { raw_tx: rpc_tx }))
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
        &data.deployed_contracts.bridge_lockscript.code_hash,
        &data.deployed_contracts.sudt.code_hash,
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

#[post("/lock")]
pub async fn lock(
    data: web::Data<DappState>,
    args: web::Json<Value>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let args: LockArgs =
        serde_json::from_value(args.into_inner()).map_err(|e| format!("invalid args: {}", e))?;
    log::info!("lock args: {:?}", args);
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

    let data = [
        Token::Address(token_addr),
        Token::Uint(amount),
        Token::Uint(bridge_fee),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(
            hex::decode(&args.replay_resist_outpoint)
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
    let raw_transaction = make_transaction(to, nonce, input_data, gas_price, eth_value);
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

            let typescript = parse_cell(
                data.deployed_contracts
                    .light_client_cell_script
                    .cell_script
                    .as_str(),
            )
            .map_err(|e| format!("get typescript fail {:?}", e))?;

            let cell = get_live_cell_by_typescript(&mut generator.indexer_client, typescript)
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
