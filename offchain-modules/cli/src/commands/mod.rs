pub mod server;
pub mod types;
use anyhow::{anyhow, bail, Result};
use cmd_lib::run_fun;
use ethabi::Token;
use force_eth_lib::relay::ckb_relay::CKBRelayer;
use force_eth_lib::relay::eth_relay::{wait_header_sync_success, ETHRelayer};
use force_eth_lib::transfer::to_ckb::{
    approve, create_bridge_cell, dev_init, get_header_rlp, lock_eth, lock_token,
    send_eth_spv_proof_tx,
};
use force_eth_lib::transfer::to_eth::{
    burn, get_balance, get_ckb_proof_info, init_light_client, transfer_sudt, unlock,
    wait_block_submit,
};
use force_eth_lib::util::ckb_util::{build_lockscript_from_address, ETHSPVProofJson, Generator};
use force_eth_lib::util::eth_util::convert_eth_address;
use force_eth_lib::util::settings::Settings;
use log::{debug, info};
use molecule::prelude::Entity;
use rusty_receipt_proof_maker::generate_eth_proof;
use serde_json::{json, Value};
use std::convert::TryFrom;
use types::*;
use web3::types::U256;

pub async fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        SubCommand::Server(args) => server::server_handler(args),

        SubCommand::InitCkbLightContract(args) => init_ckb_light_contract_handler(args).await,
        SubCommand::DevInit(args) => dev_init_handler(args),
        SubCommand::CreateBridgeCell(args) => create_bridge_cell_handler(args),
        // transfer erc20 to ckb
        SubCommand::Approve(args) => approve_handler(args).await,
        // lock erc20 token && wait the tx is commit.
        SubCommand::LockToken(args) => lock_token_handler(args).await,

        SubCommand::LockEth(args) => lock_eth_handler(args).await,
        // parse eth receipt proof from tx_hash.
        // SubCommand::GenerateEthProof(args) => generate_eth_proof_handler(args).await,
        // verify eth receipt proof && mint new token
        SubCommand::Mint(args) => mint_handler(args).await,
        SubCommand::TransferToCkb(args) => transfer_to_ckb_handler(args),
        // transfer erc20 from ckb
        SubCommand::Burn(args) => burn_handler(args),
        // parse ckb spv proof from tx_hash.
        SubCommand::GenerateCkbProof(args) => generate_ckb_proof_handler(args),
        // verify ckb spv proof && unlock erc20 token.
        SubCommand::Unlock(args) => unlock_handler(args).await,
        SubCommand::TransferFromCkb(args) => transfer_from_ckb_handler(args).await,
        SubCommand::TransferSudt(args) => transfer_sudt_handler(args),
        SubCommand::QuerySudtBlance(args) => query_sudt_balance_handler(args),

        SubCommand::EthRelay(args) => eth_relay_handler(args).await,
        SubCommand::CkbRelay(args) => ckb_relay_handler(args).await,
    }
}

pub async fn init_ckb_light_contract_handler(args: InitCkbLightContractArgs) -> Result<()> {
    let settings = Settings::new(&args.config_path)?;
    let eth_ckb_chain_addr = convert_eth_address(&settings.eth_ckb_chain_addr)?;
    let hash = init_light_client(
        args.ckb_rpc_url,
        args.indexer_url,
        args.eth_rpc_url,
        args.init_height,
        args.finalized_gc,
        args.canonical_gc,
        args.gas_price,
        eth_ckb_chain_addr,
        args.private_key_path,
        args.wait,
    )
    .await?;
    println!("init tx_hash: {:?}", &hash);
    Ok(())
}

pub fn dev_init_handler(args: DevInitArgs) -> Result<()> {
    if std::path::Path::new(&args.config_path).exists() && !args.force {
        bail!(
            "force-bridge-eth config already exists at {}, use `-f` in command if you want to overwrite it",
            &args.config_path
        );
    }
    dev_init(
        args.config_path,
        args.rpc_url,
        args.indexer_url,
        args.private_key_path,
        args.bridge_typescript_path,
        args.bridge_lockscript_path,
        args.light_client_typescript_path,
        args.light_client_lockscript_path,
        args.recipient_typescript_path,
        args.sudt_path,
    )
}

pub fn create_bridge_cell_handler(args: CreateBridgeCellArgs) -> Result<()> {
    let outpoint_hex = create_bridge_cell(
        args.config_path,
        args.rpc_url,
        args.indexer_url,
        args.private_key_path,
        args.tx_fee,
        args.capacity,
        args.eth_token_address,
        args.recipient_address.clone(),
        args.bridge_fee,
    )?;
    info!(
        "create bridge cell successfully for {}, outpoint: {}",
        &args.recipient_address, &outpoint_hex
    );
    println!("{}", json!({ "outpoint": outpoint_hex }));
    Ok(())
}

pub async fn approve_handler(args: ApproveArgs) -> Result<()> {
    debug!("approve_handler args: {:?}", &args);
    let settings = Settings::new(&args.config_path)?;
    let from = convert_eth_address(&settings.eth_token_locker_addr)?;
    let to = convert_eth_address(&args.erc20_addr)?;
    let hash = approve(
        from,
        to,
        args.rpc_url,
        args.private_key_path,
        args.gas_price,
        args.wait,
    )
    .await
    .map_err(|e| anyhow!("Failed to call approve. {:?}", e))?;
    println!("approve tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn lock_token_handler(args: LockTokenArgs) -> Result<()> {
    debug!("lock_handler args: {:?}", &args);
    let settings = Settings::new(&args.config_path)?;
    let to = convert_eth_address(&settings.eth_token_locker_addr)?;
    let token_addr = convert_eth_address(&args.token)?;
    let recipient_lockscript = build_lockscript_from_address(args.ckb_recipient_address.as_str())?;
    let data = [
        Token::Address(token_addr),
        Token::Uint(U256::from(args.amount)),
        Token::Uint(U256::from(args.bridge_fee)),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(hex::decode(args.replay_resist_outpoint)?),
        Token::Bytes(args.sudt_extra_data.as_bytes().to_vec()),
    ];
    let hash = lock_token(
        to,
        args.rpc_url,
        args.private_key_path,
        args.gas_price,
        &data,
        args.wait,
    )
    .await
    .map_err(|e| anyhow!("Failed to call lock_token. {:?}", e))?;
    println!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn lock_eth_handler(args: LockEthArgs) -> Result<()> {
    debug!("lock_handler args: {:?}", &args);
    let settings = Settings::new(&args.config_path)?;
    let to = convert_eth_address(&settings.eth_token_locker_addr)?;
    let recipient_lockscript = build_lockscript_from_address(args.ckb_recipient_address.as_str())?;
    let data = [
        Token::Uint(U256::from(args.bridge_fee)),
        Token::Bytes(recipient_lockscript.as_slice().to_vec()),
        Token::Bytes(hex::decode(args.replay_resist_outpoint)?),
        Token::Bytes(args.sudt_extra_data.as_bytes().to_vec()),
    ];
    let hash = lock_eth(
        to,
        args.rpc_url.clone(),
        args.private_key_path,
        &data,
        args.gas_price,
        U256::from(args.amount),
        args.wait,
    )
    .await
    .map_err(|e| anyhow!("Failed to call lock_eth. {:?}", e))?;
    println!("lock eth tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn mint_handler(args: MintArgs) -> Result<()> {
    debug!("mint_handler args: {:?}", &args);
    let eth_spv_proof = generate_eth_proof(args.hash.clone(), args.eth_rpc_url.clone())
        .map_err(|e| anyhow!("Failed to generate eth proof. {:?}", e))?;
    let header_rlp = get_header_rlp(args.eth_rpc_url.clone(), eth_spv_proof.block_hash).await?;
    let hash_str = args.hash.clone();
    let log_index = eth_spv_proof.log_index;
    let network = args.eth_rpc_url;
    let proof_hex = run_fun! {
    node eth-proof/index.js proof --hash ${hash_str} --index ${log_index} --network ${network}}
    .unwrap();
    let proof_json: Value = serde_json::from_str(&proof_hex.clone()).unwrap();
    info!("generate proof json: {:?}", proof_json);

    let settings = Settings::new(&args.config_path)?;
    let eth_proof = ETHSPVProofJson {
        log_index: u64::try_from(log_index).unwrap(),
        log_entry_data: String::from(proof_json["log_data"].as_str().unwrap()),
        receipt_index: eth_spv_proof.receipt_index,
        receipt_data: String::from(proof_json["receipt_data"].as_str().unwrap()),
        header_data: header_rlp.clone(),
        proof: vec![proof_json["proof"][0].as_str().unwrap().to_owned()],
        token: eth_spv_proof.token,
        lock_amount: eth_spv_proof.lock_amount,
        recipient_lockscript: eth_spv_proof.recipient_lockscript,
        sudt_extra_data: eth_spv_proof.sudt_extra_data,
        bridge_fee: eth_spv_proof.bridge_fee,
        replay_resist_outpoint: eth_spv_proof.replay_resist_outpoint,
        eth_address: convert_eth_address(&settings.eth_token_locker_addr)?,
    };
    let mut generator = Generator::new(args.ckb_rpc_url, args.indexer_url, settings)
        .map_err(|e| anyhow::anyhow!(e))?;
    wait_header_sync_success(&mut generator, args.config_path.clone(), header_rlp.clone())?;
    let tx_hash = send_eth_spv_proof_tx(
        &mut generator,
        args.config_path,
        &eth_proof,
        args.private_key_path,
    )
    .await?;
    println!("mint erc20 token on ckb. tx_hash: {}", &tx_hash);
    Ok(())
}

pub fn transfer_to_ckb_handler(args: TransferToCkbArgs) -> Result<()> {
    debug!("transfer_to_ckb_handler args: {:?}", &args);
    todo!()
}

pub fn burn_handler(args: BurnArgs) -> Result<()> {
    debug!("burn_handler args: {:?}", &args);
    let token_addr = convert_eth_address(&args.token_addr)?;
    let receive_addr = convert_eth_address(&args.receive_addr)?;
    let settings = Settings::new(&args.config_path)?;
    let lock_contract_addr = convert_eth_address(&settings.eth_token_locker_addr)?;
    let ckb_tx_hash = burn(
        args.private_key_path,
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        &args.config_path,
        args.tx_fee,
        args.unlock_fee,
        args.burn_amount,
        token_addr,
        receive_addr,
        lock_contract_addr,
    )?;
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);
    Ok(())
}

pub fn generate_ckb_proof_handler(args: GenerateCkbProofArgs) -> Result<()> {
    debug!("generate_ckb_proof_handler args: {:?}", &args);
    let (header, tx) = get_ckb_proof_info(&args.tx_hash, args.ckb_rpc_url)?;
    println!("headers : {:?}", header);
    println!("tx : {:?}", tx);
    Ok(())
}

pub async fn unlock_handler(args: UnlockArgs) -> Result<()> {
    debug!("unlock_handler args: {:?}", &args);
    let to = convert_eth_address(&args.to)?;
    let result = unlock(
        to,
        args.private_key_path,
        args.tx_proof,
        args.tx_info,
        args.eth_rpc_url,
        args.gas_price,
        args.wait,
    )
    .await?;
    println!("unlock tx hash : {:?}", result);
    Ok(())
}

pub async fn transfer_from_ckb_handler(args: TransferFromCkbArgs) -> Result<()> {
    debug!("transfer_from_ckb_handler args: {:?}", &args);
    let token_addr = convert_eth_address(&args.token_addr)?;
    let receive_addr = convert_eth_address(&args.receive_addr)?;
    let settings = Settings::new(&args.config_path)?;
    let lock_contract_addr = convert_eth_address(&settings.eth_token_locker_addr)?;

    let ckb_tx_hash = burn(
        args.ckb_privkey_path,
        args.ckb_rpc_url.clone(),
        args.indexer_rpc_url,
        &args.config_path,
        args.tx_fee,
        args.unlock_fee,
        args.burn_amount,
        token_addr,
        receive_addr,
        lock_contract_addr,
    )?;
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);

    let (tx_proof, tx_info) = get_ckb_proof_info(&ckb_tx_hash, args.ckb_rpc_url.clone())?;

    let settings = Settings::new(&args.config_path)?;
    let light_client = convert_eth_address(&settings.eth_ckb_chain_addr)?;
    let to = convert_eth_address(&settings.eth_token_locker_addr)?;

    wait_block_submit(
        args.eth_rpc_url.clone(),
        args.ckb_rpc_url,
        light_client,
        ckb_tx_hash,
    )
    .await?;
    let result = unlock(
        to,
        args.eth_privkey_path,
        tx_proof,
        tx_info,
        args.eth_rpc_url,
        args.gas_price,
        args.wait,
    )
    .await?;
    println!("unlock tx hash : {:?}", result);
    Ok(())
}
pub fn transfer_sudt_handler(args: TransferSudtArgs) -> Result<()> {
    debug!("mock_transfer_sudt_handler args: {:?}", &args);
    let token_addr = convert_eth_address(&args.token_addr)?;
    let settings = Settings::new(&args.config_path)?;
    let lock_contract_addr = convert_eth_address(&settings.eth_token_locker_addr)?;
    transfer_sudt(
        args.private_key_path,
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.config_path,
        args.to_addr,
        args.tx_fee,
        args.ckb_amount,
        args.sudt_amount,
        token_addr,
        lock_contract_addr,
    )?;
    Ok(())
}

pub fn query_sudt_balance_handler(args: SudtGetBalanceArgs) -> Result<()> {
    debug!("query sudt balance handler args: {:?}", &args);
    let token_addr = convert_eth_address(&args.token_addr)?;
    let settings = Settings::new(&args.config_path)?;
    let lock_contract_addr = convert_eth_address(&settings.eth_token_locker_addr)?;

    let result = get_balance(
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.config_path,
        args.addr,
        token_addr,
        lock_contract_addr,
    )?;
    info!("sudt balance is {} ", result);
    Ok(())
}

pub async fn eth_relay_handler(args: EthRelayArgs) -> Result<()> {
    debug!("eth_relay_handler args: {:?}", &args);
    let mut eth_relayer = ETHRelayer::new(
        args.config_path,
        args.ckb_rpc_url,
        args.eth_rpc_url,
        args.indexer_rpc_url,
        args.private_key_path,
        args.proof_data_path,
    )?;
    loop {
        let res = eth_relayer.start().await;
        if let Err(err) = res {
            println!("An error occurred during the eth relay. Err: {:?}", err)
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub async fn ckb_relay_handler(args: CkbRelayArgs) -> Result<()> {
    debug!("ckb_relay_handler args: {:?}", &args);
    let settings = Settings::new(&args.config_path)?;
    let to = convert_eth_address(&settings.eth_ckb_chain_addr)?;
    let mut ckb_relayer = CKBRelayer::new(
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.eth_rpc_url,
        to,
        args.private_key_path,
        args.gas_price,
    )?;
    loop {
        ckb_relayer.start(args.per_amount).await?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
