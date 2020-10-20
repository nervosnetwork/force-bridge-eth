use clap::Clap;

#[derive(Clap, Clone, Debug)]
#[clap(version = "0.1", author = "LeonLi000 <matrix.skygirl@gmail.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    TransferToCkb(TransferToCkbArgs),
    Approve(ApproveArgs),
    Lock(LockArgs),
    ParseEthProof(ParseEthProofArgs),
    WaitEthBlockSafe(WaitEthBlockSafeArgs),
    VerifyEthSpvProof(VerifyEthSpvProofArgs),
    Mint(MintArgs),
    TransferFromCkb(TransferFromCkbArgs),
    Burn(BurnArgs),
    ParseCkbProof(ParseCkbProofArgs),
    WaitCkbBlockSafe(WaitCkbBlockSafeArgs),
    VerifyCkbSpvProof(VerifyCkbSpvProofArgs),
    Unlock(UnlockArgs),
    EthRelay(EthRelayArgs),
    CkbRelay(CkbRelayArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct TransferToCkbArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct ApproveArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct LockArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct ParseEthProofArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct WaitEthBlockSafeArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct VerifyEthSpvProofArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct MintArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct TransferFromCkbArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct BurnArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct ParseCkbProofArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct WaitCkbBlockSafeArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct VerifyCkbSpvProofArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct UnlockArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct EthRelayArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct CkbRelayArgs {

}
