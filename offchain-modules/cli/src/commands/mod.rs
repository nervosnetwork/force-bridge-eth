pub mod types;
use anyhow::{Error, Result};
use ethabi::Token;
use force_eth_lib::transfer::to_ckb::{
    approve, dev_init, get_header_rlp, lock_eth, lock_token, send_eth_spv_proof_tx,
};
use force_eth_lib::transfer::to_eth::burn;
use force_eth_lib::util::ckb_util::{ETHSPVProofJson, Generator};
use force_eth_lib::util::settings::Settings;
use log::debug;
use rusty_receipt_proof_maker::generate_eth_proof;
use std::convert::TryFrom;
use types::*;
use web3::types::{H160, H256, U256};

pub const ETH_ADDRESS_LENGTH: usize = 40;

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
        args.typescript_path,
        args.lockscript_path,
        args.sudt_path,
    )
}

pub async fn approve_handler(args: ApproveArgs) -> Result<()> {
    debug!("approve_handler args: {:?}", &args);
    if args.from.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid from address"));
    }
    if args.to.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid to address"));
    }
    let from: H160 = H160::from_slice(hex::decode(args.from)?.as_slice());
    let to = H160::from_slice(hex::decode(args.to)?.as_slice());
    let hash = approve(from, to, args.rpc_url, args.private_key_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call approve. {:?}", e))?;
    println!("approve tx_hash: {:?}", &hash);
    Ok(())
}

pub async fn lock_token_handler(args: LockTokenArgs) -> Result<()> {
    debug!("lock_handler args: {:?}", &args);
    if args.from.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid from address"));
    }
    if args.to.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid to address"));
    }
    if args.token.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid token address"));
    }
    let from: H160 = H160::from_slice(hex::decode(args.from)?.as_slice());
    let to = H160::from_slice(hex::decode(args.to)?.as_slice());
    let data = [
        Token::Address(H160::from_slice(hex::decode(args.token)?.as_slice())),
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
    if args.from.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid from address"));
    }
    if args.to.len() != ETH_ADDRESS_LENGTH {
        return Err(Error::msg("invalid to address"));
    }
    let from: H160 = H160::from_slice(hex::decode(args.from)?.as_slice());
    let to = H160::from_slice(hex::decode(args.to)?.as_slice());
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
    let header_rlp = get_header_rlp(
        args.eth_rpc_url,
        H256::from_slice(hex::decode(args.hash.clone())?.as_slice()),
    )
    .await?;
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
    let tx_hash = send_eth_spv_proof_tx(&mut generator, &eth_proof, args.private_key_path).await?;
    println!("mint erc20 token on ckb. tx_hash: {}", &tx_hash);
    Ok(())
}

pub fn transfer_to_ckb_handler(args: TransferToCkbArgs) -> Result<()> {
    debug!("transfer_to_ckb_handler args: {:?}", &args);
    todo!()
}

pub fn burn_handler(args: BurnArgs) -> Result<()> {
    debug!("burn_handler args: {:?}", &args);
    let ckb_tx_hash = burn(args.private_key_path, args.rpc_url).unwrap();
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);
    todo!()
}

pub fn generate_ckb_proof_handler(args: GenerateCkbProofArgs) -> Result<()> {
    debug!("generate_ckb_proof_handler args: {:?}", &args);
    todo!()
}

pub fn unlock_handler(args: UnlockArgs) -> Result<()> {
    debug!("unlock_handler args: {:?}", &args);
    todo!()
}

pub fn transfer_from_ckb_handler(args: TransferFromCkbArgs) -> Result<()> {
    debug!("transfer_from_ckb_handler args: {:?}", &args);
    todo!()
}
