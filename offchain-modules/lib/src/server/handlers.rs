use super::error::RpcError;
use super::state::DappState;
use super::types::*;
use crate::transfer::to_ckb::create_bridge_cell;
use crate::util::ckb_util::{build_lockscript_from_address, parse_cell, parse_main_chain_headers};
use crate::util::eth_util::{
    build_lock_eth_payload, build_lock_token_payload, convert_eth_address, make_transaction,
    rlp_transaction, Web3Client,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use ckb_jsonrpc_types::{Uint128, Uint64};
use ckb_sdk::{Address, HumanCapacity};
use ckb_types::packed::Script;
use ethabi::Token;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use molecule::prelude::Entity;
use serde_json::json;
use std::str::FromStr;
use web3::types::U256;

#[post("/get_or_create_bridge_cell")]
pub async fn get_or_create_bridge_cell(
    data: web::Data<DappState>,
    args: web::Json<CreateBridgeCellArgs>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let tx_fee = "0.1".to_string();
    let capacity = "283".to_string();
    let outpoint = create_bridge_cell(
        data.config_path.clone(),
        data.ckb_rpc_url.clone(),
        data.indexer_url.clone(),
        data.private_key_path.clone(),
        tx_fee,
        capacity,
        args.eth_token_address.clone(),
        args.recipient_address.clone(),
        args.bridge_fee.into(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(CreateBridgeCellResponse { outpoint }))
}

#[post("/burn")]
pub async fn burn(
    data: web::Data<DappState>,
    args: web::Json<BurnArgs>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let from_lockscript = Script::from(
        Address::from_str(args.from_lockscript_addr.as_str())
            .map_err(|err| format!("ckb_address to script fail: {}", err))?
            .payload(),
    );
    let token_address = convert_eth_address(args.token_address.as_str())?;
    let lock_contract_address = convert_eth_address(data.settings.eth_token_locker_addr.as_str())?;
    let recipient_address = convert_eth_address(args.recipient_address.as_str())?;

    let mut generator = data.get_generator().await?;

    let tx_fee: u64 = HumanCapacity::from_str(&args.tx_fee)?.into();

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
    Ok(HttpResponse::Ok().json(BurnResult { raw_tx: rpc_tx }))
}

#[post("/get_sudt_balance")]
pub async fn get_sudt_balance(
    data: web::Data<DappState>,
    args: web::Json<GetSudtBalanceArgs>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let token_address = convert_eth_address(args.token_address.as_str())
        .map_err(|e| format!("token address parse fail: {}", e))?;
    let lock_contract_address = convert_eth_address(data.settings.eth_token_locker_addr.as_str())
        .map_err(|e| format!("lock contract address parse fail: {}", e))?;

    let mut generator = data.get_generator().await?;

    let balance: Uint128 = generator
        .get_sudt_balance(args.address.clone(), token_address, lock_contract_address)
        .map_err(|e| format!("get_sudt_balance fail, err: {}", e))?
        .into();
    Ok(HttpResponse::Ok().json(json! ({
        "balance": balance,
    })))
}

#[post("/lock")]
pub async fn lock(
    data: web::Data<DappState>,
    args: web::Json<LockArgs>,
) -> actix_web::Result<HttpResponse, RpcError> {
    let to = convert_eth_address(data.settings.eth_token_locker_addr.as_str())
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
    args: web::Json<GetBestBlockHeightArgs>,
) -> actix_web::Result<HttpResponse, RpcError> {
    match args.chain.as_str() {
        "ckb" => {
            let contract_address = convert_eth_address(&data.settings.eth_ckb_chain_addr)
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

            let typescript =
                parse_cell(data.settings.light_client_cell_script.cell_script.as_str())
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
    HttpResponse::Ok().json(&data.settings)
}
