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
    VerifyEthSpvProof(VerifyEthSpvProofArgs),
    TransferFromCkb(TransferFromCkbArgs),
    Burn(BurnArgs),
    ParseCkbProof(ParseCkbProofArgs),
    Unlock(UnlockArgs),
    EthRelay(EthRelayArgs),
    CkbRelay(CkbRelayArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct TransferToCkbArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct ApproveArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "https://localhost:8545")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct LockArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "https://localhost:8545")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct ParseEthProofArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct VerifyEthSpvProofArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct TransferFromCkbArgs {

}

#[derive(Clap, Clone, Debug)]
pub struct BurnArgs {
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "https://localhost:8114")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct ParseCkbProofArgs {

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