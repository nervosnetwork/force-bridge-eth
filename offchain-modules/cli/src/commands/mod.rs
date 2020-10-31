pub mod types;
use anyhow::Result;
use ethabi::Token;
use force_eth_lib::relay::ckb_relay::CKBRelayer;
use force_eth_lib::relay::eth_relay::ETHRelayer;
use force_eth_lib::transfer::to_ckb::{
    approve, dev_init, get_header_rlp, lock_eth, lock_token, send_eth_spv_proof_tx,
};
use force_eth_lib::transfer::to_eth::{burn, get_balance, get_ckb_proof_info, transfer_sudt};
use force_eth_lib::util::ckb_util::{ETHSPVProofJson, Generator};
use force_eth_lib::util::eth_util::convert_eth_address;
use force_eth_lib::util::settings::Settings;
use log::{debug, info};
use rusty_receipt_proof_maker::generate_eth_proof;
use std::convert::TryFrom;
use types::*;
use web3::types::{H256, U256};

pub async fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        SubCommand::DevInit(args) => dev_init_handler(args),
        // transfer erc20 to ckb
        SubCommand::Approve(args) => approve_handler(args).await,
        // lock erc20 token && wait the tx is commit.
        SubCommand::LockToken(args) => lock_token_handler(args).await,

        SubCommand::LockEth(args) => lock_eth_handler(args).await,
        // parse eth receipt proof from tx_hash.
        SubCommand::GenerateEthProof(args) => generate_eth_proof_handler(args).await,
        // verify eth receipt proof && mint new token
        SubCommand::Mint(args) => mint_handler(args).await,
        SubCommand::TransferToCkb(args) => transfer_to_ckb_handler(args),
        // transfer erc20 from ckb
        SubCommand::Burn(args) => burn_handler(args),
        // parse ckb spv proof from tx_hash.
        SubCommand::GenerateCkbProof(args) => generate_ckb_proof_handler(args),
        // verify ckb spv proof && unlock erc20 token.
        SubCommand::Unlock(args) => unlock_handler(args),
        SubCommand::TransferFromCkb(args) => transfer_from_ckb_handler(args),
        SubCommand::QuerySudtBlance(args) => query_sudt_balance_handler(args),

        SubCommand::EthRelay(args) => eth_relay_handler(args).await,
        SubCommand::CkbRelay(args) => ckb_relay_handler(args).await,
    }
}

pub fn dev_init_handler(args: DevInitArgs) -> Result<()> {
    if std::path::Path::new(&args.config_path).exists() && !args.force {
        return Err(anyhow::anyhow!(
            "force-bridge-eth config already exists at {}, use `-f` in command if you want to overwrite it",
            &args.config_path
        ));
    }
    dev_init(
        args.config_path,
        args.rpc_url,
        args.indexer_url,
        args.private_key_path,
        args.bridge_typescript_path,
        args.bridge_lockscript_path,
        args.light_client_typescript_path,
        args.eth_recipient_typescript_path,
        args.sudt_path,
    )
}

pub async fn approve_handler(args: ApproveArgs) -> Result<()> {
    debug!("approve_handler args: {:?}", &args);
    let from = convert_eth_address(&args.from)?;
    let to = convert_eth_address(&args.to)?;
    let hash = approve(from, to, args.rpc_url, args.private_key_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call approve. {:?}", e))?;
    println!("approve tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn lock_token_handler(args: LockTokenArgs) -> Result<()> {
    debug!("lock_handler args: {:?}", &args);
    let from = convert_eth_address(&args.from)?;
    let to = convert_eth_address(&args.to)?;
    let token_addr = convert_eth_address(&args.token)?;
    let data = [
        Token::Address(token_addr),
        Token::Uint(U256::from(args.amount)),
        Token::String(args.ckb_address),
    ];
    let hash = lock_token(from, to, args.rpc_url, args.private_key_path, &data)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call lock_token. {:?}", e))?;
    println!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn lock_eth_handler(args: LockEthArgs) -> Result<()> {
    debug!("lock_handler args: {:?}", &args);
    let from = convert_eth_address(&args.from)?;
    let to = convert_eth_address(&args.to)?;
    let data = [Token::String(args.ckb_address)];
    let hash = lock_eth(
        from,
        to,
        args.rpc_url.clone(),
        args.private_key_path,
        &data,
        U256::from(args.amount),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to call lock_eth. {:?}", e))?;
    println!("lock erc20 token tx_hash: {:?}", &hash);
    let eth_spv_proof =
        generate_eth_proof(format!("0x{}", hex::encode(hash.0)), args.rpc_url.clone())
            .map_err(|e| anyhow::anyhow!("Failed to generate eth proof. {:?}", e))?;
    println!(
        "generate eth proof with hash: {:?}, eth_spv_proof: {:?}",
        hash.clone(),
        eth_spv_proof
    );
    Ok(())
}

pub async fn generate_eth_proof_handler(args: GenerateEthProofArgs) -> Result<()> {
    debug!("generate_eth_proof_handler args: {:?}", &args);
    let eth_spv_proof = generate_eth_proof(args.hash.clone(), args.rpc_url.clone())
        .map_err(|e| anyhow::anyhow!("Failed to generate eth proof. {:?}", e))?;
    println!(
        "generate eth proof with hash: {:?}, eth_spv_proof: {:?}",
        args.hash, eth_spv_proof
    );
    let header_rlp = get_header_rlp(
        args.rpc_url,
        H256::from_slice(
            hex::decode(args.hash.clone())
                .map_err(|e| anyhow::anyhow!("invalid cmd args `hash`. FromHexError: {:?}", e))?
                .as_slice(),
        ),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to get header_rlp. {:?}", e))?;
    println!(
        "generate eth proof with hash: {:?}, header_rlp: {:?}",
        args.hash, header_rlp
    );
    Ok(())
}

pub async fn mint_handler(args: MintArgs) -> Result<()> {
    debug!("mint_handler args: {:?}", &args);
    let eth_spv_proof = generate_eth_proof(args.hash.clone(), args.eth_rpc_url.clone())
        .map_err(|e| anyhow::anyhow!("Failed to generate eth proof. {:?}", e))?;
    let header_rlp = get_header_rlp(args.eth_rpc_url, eth_spv_proof.block_hash).await?;
    let eth_proof = ETHSPVProofJson {
        log_index: u64::try_from(eth_spv_proof.log_index).unwrap(),
        log_entry_data: eth_spv_proof.log_entry_data,
        receipt_index: eth_spv_proof.receipt_index,
        receipt_data: eth_spv_proof.receipt_data,
        header_data: header_rlp,
        proof: vec![eth_spv_proof.proof.into_bytes()],
        token: eth_spv_proof.token,
        lock_amount: eth_spv_proof.lock_amount,
        ckb_recipient: eth_spv_proof.ckb_recipient,
    };
    let settings = Settings::new(&args.config_path)?;
    let mut generator = Generator::new(args.ckb_rpc_url, args.indexer_url, settings)
        .map_err(|e| anyhow::anyhow!(e))?;
    let tx_hash =
        send_eth_spv_proof_tx(&mut generator, &eth_proof, args.private_key_path, args.cell).await?;
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
    let ckb_tx_hash = burn(
        args.private_key_path,
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.config_path,
        args.tx_fee,
        args.amount,
        token_addr,
        receive_addr,
    )?;
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);
    todo!()
}

pub fn generate_ckb_proof_handler(args: GenerateCkbProofArgs) -> Result<()> {
    debug!("generate_ckb_proof_handler args: {:?}", &args);
    let (header, tx) = get_ckb_proof_info(&args.tx_hash, args.ckb_rpc_url)?;
    println!("headers : {:?}", header);
    println!("tx : {:?}", tx);
    Ok(())
}

pub fn unlock_handler(args: UnlockArgs) -> Result<()> {
    debug!("unlock_handler args: {:?}", &args);
    todo!()
}

pub fn transfer_from_ckb_handler(args: TransferFromCkbArgs) -> Result<()> {
    debug!("transfer_from_ckb_handler args: {:?}", &args);
    let token_addr = convert_eth_address(&args.token_addr)?;
    transfer_sudt(
        args.private_key_path,
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.config_path,
        args.to_addr,
        args.tx_fee,
        args.sudt_amount,
        token_addr,
    )?;
    Ok(())
}

pub fn query_sudt_balance_handler(args: SudtGetBalanceArgs) -> Result<()> {
    debug!("query sudt balance handler args: {:?}", &args);
    let token_addr = convert_eth_address(&args.token_addr)?;

    let result = get_balance(
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.config_path,
        args.addr,
        token_addr,
    )?;
    info!("sudt balance is {} ", result);
    Ok(())
}

pub async fn eth_relay_handler(args: EthRelayArgs) -> Result<()> {
    debug!("eth_relay_handler args: {:?}", &args);
    let mut eth_relayer = ETHRelayer::new(
        args.config_path,
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.eth_rpc_url,
        args.private_key_path,
        args.proof_data_path,
        args.cell,
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
    let from = convert_eth_address(&args.from)?;
    let to = convert_eth_address(&args.to)?;
    let mut ckb_relayer = CKBRelayer::new(
        args.ckb_rpc_url,
        args.indexer_rpc_url,
        args.eth_rpc_url,
        from,
        to,
        args.private_key_path,
    )?;
    loop {
        ckb_relayer.start().await?;
        std::thread::sleep(std::time::Duration::from_secs(10 * 60));
    }
}
