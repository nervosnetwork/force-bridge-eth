use anyhow::{anyhow, Result};
use force_eth_lib::relay::ckb_relay::CKBRelayer;
use force_eth_lib::relay::eth_relay::ETHRelayer;
use force_eth_lib::transfer::to_ckb::{
    self, approve, create_bridge_cell, generate_eth_spv_proof_json, lock_eth, lock_token,
    send_eth_spv_proof_tx,
};
use force_eth_lib::transfer::to_eth::{
    burn, get_balance, get_ckb_proof_info, init_light_client, transfer_sudt, unlock,
    wait_block_submit,
};
use force_eth_lib::util::ckb_util::{parse_privkey_path, Generator};
use force_eth_lib::util::config;
use force_eth_lib::util::config::ForceConfig;
use force_eth_lib::util::eth_util::{convert_eth_address, parse_private_key};
use log::{debug, error, info};
use serde_json::json;
use shellexpand::tilde;
use types::*;

pub mod server;
pub mod types;

pub async fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        SubCommand::Server(args) => server::server_handler(args).await,
        SubCommand::InitCkbLightContract(args) => init_ckb_light_contract_handler(args).await,
        SubCommand::Init(args) => init_config(args).await,
        SubCommand::DeployCKB(args) => deploy_ckb(args).await,
        SubCommand::CreateBridgeCell(args) => create_bridge_cell_handler(args).await,
        // transfer erc20 to ckb
        SubCommand::Approve(args) => approve_handler(args).await,
        // lock erc20 token && wait the tx is commit.
        SubCommand::LockToken(args) => lock_token_handler(args).await,
        SubCommand::LockEth(args) => lock_eth_handler(args).await,
        // parse eth receipt proof from tx_hash.
        // SubCommand::GenerateEthProof(args) => generate_eth_proof_handler(args).await,
        // verify eth receipt proof && mint new token
        SubCommand::Mint(args) => mint_handler(args).await,
        SubCommand::TransferToCkb(args) => transfer_to_ckb_handler(args).await,
        // transfer erc20 from ckb
        SubCommand::Burn(args) => burn_handler(args).await,
        // parse ckb spv proof from tx_hash.
        SubCommand::GenerateCkbProof(args) => generate_ckb_proof_handler(args).await,
        // verify ckb spv proof && unlock erc20 token.
        SubCommand::Unlock(args) => unlock_handler(args).await,
        SubCommand::TransferFromCkb(args) => transfer_from_ckb_handler(args).await,
        SubCommand::TransferSudt(args) => transfer_sudt_handler(args).await,
        SubCommand::QuerySudtBlance(args) => query_sudt_balance_handler(args).await,
        SubCommand::EthRelay(args) => eth_relay_handler(args).await,
        SubCommand::CkbRelay(args) => ckb_relay_handler(args).await,
    }
}

pub async fn init_ckb_light_contract_handler(args: InitCkbLightContractArgs) -> Result<()> {
    let hash = init_light_client(
        args.config_path,
        args.network,
        args.private_key_path,
        args.init_height,
        args.finalized_gc,
        args.canonical_gc,
        args.gas_price,
        args.wait,
    )
    .await?;
    println!("init tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn init_config(args: InitArgs) -> Result<()> {
    config::init_config(
        args.force,
        args.project_path,
        args.config_path,
        args.default_network,
        args.ckb_rpc_url,
        args.ckb_indexer_url,
        args.ethereum_rpc_url,
    )
    .await
}

pub async fn deploy_ckb(args: DeployCKBArgs) -> Result<()> {
    to_ckb::deploy_ckb(args.config_path, args.network, args.eth_dag_path).await
}

pub async fn create_bridge_cell_handler(args: CreateBridgeCellArgs) -> Result<()> {
    let outpoint_hex = create_bridge_cell(
        args.config_path,
        args.network,
        args.private_key_path,
        args.tx_fee,
        args.capacity,
        args.eth_token_address,
        args.recipient_address.clone(),
        args.bridge_fee,
        1,
    )
    .await?;
    info!(
        "create bridge cell successfully for {}, outpoint: {:?}",
        &args.recipient_address, &outpoint_hex
    );
    println!("{}", json!({ "outpoint": outpoint_hex[0] }));
    Ok(())
}

pub async fn approve_handler(args: ApproveArgs) -> Result<()> {
    debug!("approve_handler args: {:?}", &args);
    let hash = approve(
        args.config_path,
        args.network,
        args.private_key_path,
        args.erc20_addr,
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
    let hash = lock_token(
        args.config_path,
        args.network,
        args.private_key_path,
        args.token,
        args.ckb_recipient_address,
        args.amount,
        args.bridge_fee,
        args.sudt_extra_data,
        args.replay_resist_outpoint,
        args.gas_price,
        args.wait,
    )
    .await
    .map_err(|e| anyhow!("Failed to call lock_token. {:?}", e))?;
    println!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn lock_eth_handler(args: LockEthArgs) -> Result<()> {
    debug!("lock_handler args: {:?}", &args);
    let hash = lock_eth(
        args.config_path,
        args.network,
        args.private_key_path,
        args.ckb_recipient_address,
        args.amount,
        args.bridge_fee,
        args.sudt_extra_data.unwrap_or_default(),
        args.replay_resist_outpoint,
        args.gas_price,
        args.wait,
    )
    .await
    .map_err(|e| anyhow!("Failed to call lock_eth. {:?}", e))?;
    println!("lock eth tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn mint_handler(args: MintArgs) -> Result<()> {
    debug!("mint_handler args: {:?}", &args);
    let force_config = ForceConfig::new(args.config_path.as_str())?;
    let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&args.network)?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&args.network)?;
    let ckb_indexer_url = force_config.get_ckb_indexer_url(&args.network)?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let eth_proof = generate_eth_spv_proof_json(
        args.hash.clone(),
        ethereum_rpc_url.clone(),
        deployed_contracts.eth_token_locker_addr.clone(),
    )
    .await?;
    let mut generator = Generator::new(ckb_rpc_url, ckb_indexer_url, deployed_contracts.clone())
        .map_err(|e| anyhow::anyhow!(e))?;
    // wait_header_sync_success(
    //     &mut generator,
    //     deployed_contracts
    //         .light_client_cell_script
    //         .cell_script
    //         .as_str(),
    //     header_rlp.clone(),
    // )
    // .await?;
    let from_privkey =
        parse_privkey_path(args.private_key_path.as_str(), &force_config, &args.network)?;
    let config_path = tilde(args.config_path.as_str()).into_owned();
    let tx_hash =
        send_eth_spv_proof_tx(&mut generator, config_path, &eth_proof, from_privkey).await?;
    println!("mint erc20 token on ckb. tx_hash: {}", &tx_hash);
    Ok(())
}

pub async fn transfer_to_ckb_handler(args: TransferToCkbArgs) -> Result<()> {
    debug!("transfer_to_ckb_handler args: {:?}", &args);
    todo!()
}

pub async fn burn_handler(args: BurnArgs) -> Result<()> {
    debug!("burn_handler args: {:?}", &args);
    let ckb_tx_hash = burn(
        args.config_path,
        args.network,
        args.private_key_path,
        args.tx_fee,
        args.unlock_fee,
        args.burn_amount,
        args.token_addr,
        args.receive_addr,
    )
    .await?;
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);
    Ok(())
}

pub async fn generate_ckb_proof_handler(args: GenerateCkbProofArgs) -> Result<()> {
    debug!("generate_ckb_proof_handler args: {:?}", &args);
    let force_config = ForceConfig::new(args.config_path.as_str())?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&args.network)?;
    let (header, tx) = get_ckb_proof_info(&args.tx_hash, ckb_rpc_url)?;
    println!("headers : {:?}", header);
    println!("tx : {:?}", tx);
    Ok(())
}

pub async fn unlock_handler(args: UnlockArgs) -> Result<()> {
    debug!("unlock_handler args: {:?}", &args);
    let result = unlock(
        args.config_path,
        args.network,
        args.private_key_path,
        args.to,
        args.tx_proof,
        args.tx_info,
        args.gas_price,
        args.wait,
    )
    .await?;
    println!("unlock tx hash : {:?}", result);
    Ok(())
}

pub async fn transfer_from_ckb_handler(args: TransferFromCkbArgs) -> Result<()> {
    debug!("transfer_from_ckb_handler args: {:?}", &args);
    let force_config = ForceConfig::new(args.config_path.as_str())?;
    let ethereum_rpc_url = force_config.get_ethereum_rpc_url(&args.network)?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&args.network)?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;

    let ckb_tx_hash = burn(
        args.config_path.clone(),
        args.network.clone(),
        args.ckb_privkey_path,
        args.tx_fee,
        args.unlock_fee,
        args.burn_amount,
        args.token_addr,
        args.receive_addr,
    )
    .await?;
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);

    let (tx_proof, tx_info) = get_ckb_proof_info(&ckb_tx_hash, ckb_rpc_url.clone())?;
    let light_client = convert_eth_address(&deployed_contracts.eth_ckb_chain_addr)?;
    let lock_contract_addr = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    wait_block_submit(
        ethereum_rpc_url.clone(),
        ckb_rpc_url,
        light_client,
        ckb_tx_hash,
        lock_contract_addr,
    )
    .await?;
    let result = unlock(
        args.config_path.clone(),
        args.network,
        args.eth_privkey_path,
        deployed_contracts.eth_token_locker_addr.clone(),
        tx_proof,
        tx_info,
        args.gas_price,
        args.wait,
    )
    .await?;
    println!("unlock tx hash : {:?}", result);
    Ok(())
}

pub async fn transfer_sudt_handler(args: TransferSudtArgs) -> Result<()> {
    debug!("mock_transfer_sudt_handler args: {:?}", &args);
    transfer_sudt(
        args.config_path,
        args.network,
        args.private_key_path,
        args.to_addr,
        args.tx_fee,
        args.ckb_amount,
        args.sudt_amount,
        args.token_addr,
    )
    .await?;
    Ok(())
}

pub async fn query_sudt_balance_handler(args: SudtGetBalanceArgs) -> Result<()> {
    debug!("query sudt balance handler args: {:?}", &args);
    let result = get_balance(args.config_path, args.network, args.addr, args.token_addr).await?;
    info!("sudt balance is {} ", result);
    Ok(())
}

pub async fn eth_relay_handler(args: EthRelayArgs) -> Result<()> {
    debug!("eth_relay_handler args: {:?}", &args);
    let config_path = tilde(args.config_path.as_str()).into_owned();
    let mut eth_relayer = ETHRelayer::new(
        config_path,
        args.network,
        args.private_key_path,
        args.proof_data_path,
    )?;
    loop {
        let res = eth_relayer.start().await;
        if let Err(err) = res {
            error!("An error occurred during the eth relay. Err: {:?}", err)
        }
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
}

pub async fn ckb_relay_handler(args: CkbRelayArgs) -> Result<()> {
    debug!("ckb_relay_handler args: {:?}", &args);
    let force_config = ForceConfig::new(args.config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let eth_rpc_url = force_config.get_ethereum_rpc_url(&args.network)?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&args.network)?;
    let ckb_indexer_url = force_config.get_ckb_indexer_url(&args.network)?;
    let priv_key = parse_private_key(&args.private_key_path, &force_config, &args.network)?;
    let mut ckb_relayer = CKBRelayer::new(
        ckb_rpc_url,
        ckb_indexer_url,
        eth_rpc_url.clone(),
        priv_key,
        deployed_contracts.eth_ckb_chain_addr.clone(),
        args.gas_price,
    )?;
    loop {
        let res = ckb_relayer
            .start(eth_rpc_url.clone(), args.per_amount)
            .await;
        if let Err(err) = res {
            error!("An error occurred during the ckb relay. Err: {:?}", err)
        }
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
}
