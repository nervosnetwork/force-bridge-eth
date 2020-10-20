pub mod types;

use anyhow::{Result};
use types::*;
use bridge_lib::transfer::to_ckb::{approve, lock};

pub fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        // transfer erc20 to ckb
        SubCommand::Approve(args) => approve_handler(args),
        SubCommand::Lock(args) => lock_handler(args),
        SubCommand::ParseEthProof(args)=> parse_eth_proof_handler(args),
        SubCommand::WaitEthBlockSafe(args)=> wait_eth_block_safe_handler(args),
        SubCommand::VerifyEthSpvProof(args) => verify_eth_spv_proof_handler(args),
        SubCommand::TransferToCkb(args) => transfer_to_ckb_handler(args),
        SubCommand::Mint(args) => mint_handler(args),
        // transfer erc20 from ckb
        SubCommand::Burn(args) => burn_handler(args),
        SubCommand::ParseCkbProof(args) => parse_ckb_proof_handler(args),
        SubCommand::WaitCkbBlockSafe(args) => wait_ckb_block_safe_handler(args),
        SubCommand::VerifyCkbSpvProof(args) => verify_ckb_spv_proof_handler(args),
        SubCommand::Unlock(args) => unlock_handler(args),
        SubCommand::TransferFromCkb(args) => transfer_from_ckb_handler(args),

        SubCommand::EthRelay(args) => eth_relay_handler(args),
        SubCommand::CkbRelay(args) => ckb_relay_handler(args),
    }
}

pub fn approve_handler(args: ApproveArgs) -> Result<()> {
    println!("approve_handler args: {:?}", &args);
    approve();
    Ok(())
}

pub fn lock_handler(args: LockArgs) -> Result<()> {
    println!("lock_handler args: {:?}", &args);
    let hash = lock();
    log::info!("lock erc20 token tx_hash: {:?}", &hash);
    Ok(())
}

pub fn parse_eth_proof_handler(args: ParseEthProofArgs) -> Result<()> {
    println!("parse_eth_proof_handler args: {:?}", &args);
    todo!()
}

pub fn wait_eth_block_safe_handler(args: WaitEthBlockSafeArgs) -> Result<()> {
    println!("wait_eth_block_safe_handler args: {:?}", &args);
    todo!()
}

pub fn verify_eth_spv_proof_handler(args: VerifyEthSpvProofArgs) -> Result<()> {
    println!("verify_eth_spv_proof_handler args: {:?}", &args);
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
    todo!()
}

pub fn parse_ckb_proof_handler(args: ParseCkbProofArgs) -> Result<()> {
    println!("parse_ckb_proof_handler args: {:?}", &args);
    todo!()
}

pub fn wait_ckb_block_safe_handler(args: WaitCkbBlockSafeArgs) -> Result<()> {
    println!("wait_ckb_block_safe_handler args: {:?}", &args);
    todo!()
}

pub fn verify_ckb_spv_proof_handler(args: VerifyCkbSpvProofArgs) -> Result<()> {
    println!("verify_ckb_spv_proof_handler args: {:?}", &args);
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

pub fn eth_relay_handler(args: EthRelayArgs) -> Result<()> {
    println!("eth_relay_handler args: {:?}", &args);
    todo!()
}

pub fn ckb_relay_handler(args: CkbRelayArgs) -> Result<()> {
    println!("ckb_relay_handler args: {:?}", &args);
    todo!()
}
