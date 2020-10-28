pub mod types;

use anyhow::Result;
use ethabi::Token;
use force_eth_lib::transfer::to_ckb::{approve, get_header_rlp, lock_eth, lock_token};
use force_eth_lib::transfer::to_eth::burn;
use types::*;
use web3::types::{H160, H256, U256};

pub fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        // transfer erc20 to ckb
        SubCommand::Approve(args) => approve_handler(args),
        // lock erc20 token && wait the tx is commit.
        SubCommand::LockToken(args) => lock_token_handler(args),

        SubCommand::LockEth(args) => lock_eth_handler(args),
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
    }
}

pub fn approve_handler(args: ApproveArgs) -> Result<()> {
    println!("approve_handler args: {:?}", &args);
    let from: H160 = H160::from_slice(hex::decode(args.from).unwrap().as_slice());
    let to = H160::from_slice(hex::decode(args.to).unwrap().as_slice());
    let hash = approve(from, to, args.rpc_url, args.private_key_path);
    println!("approve tx_hash: {:?}", &hash);
    Ok(())
}

pub fn lock_token_handler(args: LockTokenArgs) -> Result<()> {
    println!("lock_handler args: {:?}", &args);
    let from: H160 = H160::from_slice(hex::decode(args.from).unwrap().as_slice());
    let to = H160::from_slice(hex::decode(args.to).unwrap().as_slice());
    let data = [
        Token::Address(H160::from_slice(
            hex::decode(args.token).unwrap().as_slice(),
        )),
        Token::Uint(U256::from(args.amount)),
        Token::String(args.ckb_address),
    ];
    let hash = lock_token(from, to, args.rpc_url, args.private_key_path, &data);
    println!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub fn lock_eth_handler(args: LockEthArgs) -> Result<()> {
    println!("lock_handler args: {:?}", &args);
    let from: H160 = H160::from_slice(hex::decode(args.from).unwrap().as_slice());
    let to = H160::from_slice(hex::decode(args.to).unwrap().as_slice());
    let data = [Token::String(args.ckb_address)];
    let hash = lock_eth(
        from,
        to,
        args.rpc_url,
        args.private_key_path,
        &data,
        U256::from(args.amount),
    );
    println!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub fn generate_eth_proof_handler(args: GenerateEthProofArgs) -> Result<()> {
    println!("generate_eth_proof_handler args: {:?}", &args);
    let (proof, receipt_data, log_data) =
        rusty_receipt_proof_maker::generate_eth_proof(args.hash.clone(), args.rpc_url.clone())
            .expect("invalid receipt proof");
    println!(
        "generate eth proof with hash: {:?}, proof: {:?}, receipt_data: {:?}, log_data: {:?}",
        args.hash, proof, receipt_data, log_data
    );
    let header_rlp = get_header_rlp(
        args.rpc_url,
        H256::from_slice(hex::decode(args.hash.clone()).unwrap().as_slice()),
    );
    println!(
        "generate eth proof with hash: {:?}, header_rlp: {:?}",
        args.hash, header_rlp
    );
    Ok(())
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
    todo!()
}

pub fn unlock_handler(args: UnlockArgs) -> Result<()> {
    println!("unlock_handler args: {:?}", &args);
    todo!()
}

pub fn transfer_from_ckb_handler(args: TransferFromCkbArgs) -> Result<()> {
    println!("transfer_from_ckb_handler args: {:?}", &args);
    todo!()
}
