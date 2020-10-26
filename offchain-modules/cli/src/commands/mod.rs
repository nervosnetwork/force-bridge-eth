pub mod types;

use anyhow::Result;
use force_eth_lib::relay::ckb_relay::CKBRelayer;
use force_eth_lib::transfer::to_ckb::{approve, lock};
use force_eth_lib::transfer::to_eth::{burn, parse_ckb_proof};
use types::*;
use ethabi::Token;
use web3::types::{H160, U256};

pub fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        // transfer erc20 to ckb
        SubCommand::Approve(args) => approve_handler(args),
        // lock erc20 token && wait the tx is commit.
        SubCommand::Lock(args) => lock_handler(args),
        // parse eth receipt proof from tx_hash.
        SubCommand::GenerateEthProof(args) => generate_eth_proof_handler(args),
        // verify eth receipt proof && mint new token
        SubCommand::Mint(args) => mint_handler(args),
        SubCommand::TransferToCkb(args) => transfer_to_ckb_handler(args),
        // transfer erc20 from ckb
        SubCommand::Burn(args) => burn_handler(args),
        // parse ckb spv proof from tx_hash.
        SubCommand::GenerateCkbProof(args) => generate_ckb_proof_handler(args),
        // verify ckb spv proof && unlock erc20 token.
        SubCommand::Unlock(args) => unlock_handler(args),
        SubCommand::TransferFromCkb(args) => transfer_from_ckb_handler(args),

        SubCommand::EthRelay(args) => eth_relay_handler(args),
        SubCommand::CkbRelay(args) => ckb_relay_handler(args),
    }
}

pub fn approve_handler(args: ApproveArgs) -> Result<()> {
    println!("approve_handler args: {:?}", &args);
    let from: H160 = H160::from_slice(args.from.as_ref());
    let to = H160::from_slice(args.to.as_ref());
    let hash = approve(from, to, args.rpc_url, args.chain_id, args.private_key_path);
    log::info!("approve tx_hash: {:?}", &hash);
    Ok(())
}

pub fn lock_handler(args: LockArgs) -> Result<()> {
    println!("lock_handler args: {:?}", &args);
    let from: H160 = H160::from_slice(args.from.as_ref());
    let to = H160::from_slice(args.to.as_ref());
    let data = [Token::Address(H160::from_slice(args.token.as_ref())),
        Token::Uint(U256::from(args.amount)), Token::String(args.ckb_address)];
    let hash = lock(from, to, args.rpc_url, args.chain_id, args.private_key_path, &data);
    log::info!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub fn generate_eth_proof_handler(args: GenerateEthProofArgs) -> Result<()> {
    println!("generate_eth_proof_handler args: {:?}", &args);
    todo!()
}

pub fn mint_handler(args: MintArgs) -> Result<()> {
    println!("mint_handler args: {:?}", &args);
    todo!()
}

pub fn transfer_to_ckb_handler(args: TransferToCkbArgs) -> Result<()> {
    println!("transfer_to_ckb_handler args: {:?}", &args);
    todo!()
}

pub fn burn_handler(args: BurnArgs) -> Result<()> {
    println!("burn_handler args: {:?}", &args);
    let ckb_tx_hash = burn(args.private_key_path, args.rpc_url).unwrap();
    log::info!("burn erc20 token on ckb. tx_hash: {}", &ckb_tx_hash);
    todo!()
}

pub fn generate_ckb_proof_handler(args: GenerateCkbProofArgs) -> Result<()> {
    println!("generate_ckb_proof_handler args: {:?}", &args);
    let proof = parse_ckb_proof(args.tx_hash.as_str(), args.ckb_rpc_url).unwrap();
    print!("{:?}", proof);
    Ok(())
}

pub fn unlock_handler(args: UnlockArgs) -> Result<()> {
    println!("unlock_handler args: {:?}", &args);
    todo!()
}

pub fn transfer_from_ckb_handler(args: TransferFromCkbArgs) -> Result<()> {
    println!("transfer_from_ckb_handler args: {:?}", &args);
    todo!()
}

pub fn eth_relay_handler(args: EthRelayArgs) -> Result<()> {
    println!("eth_relay_handler args: {:?}", &args);
    todo!()
}

pub fn ckb_relay_handler(args: CkbRelayArgs) -> Result<()> {
    println!("ckb_relay_handler args: {:?}", &args);
    let mut ckb_relayer = CKBRelayer::new(args.ckb_rpc_url, args.indexer_rpc_url);
    ckb_relayer.start();
    Ok(())
}
