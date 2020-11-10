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
    LockToken(LockTokenArgs),
    LockEth(LockEthArgs),
    GenerateEthProof(GenerateEthProofArgs),
    Mint(MintArgs),
    TransferFromCkb(TransferFromCkbArgs),
    Burn(BurnArgs),
    GenerateCkbProof(GenerateCkbProofArgs),
    Unlock(UnlockArgs),
    EthRelay(EthRelayArgs),
    CkbRelay(CkbRelayArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct TransferToCkbArgs {}

#[derive(Clap, Clone, Debug)]
pub struct ApproveArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct LockTokenArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long)]
    pub token: String,
    #[clap(short, long)]
    pub amount: u128,
    #[clap(short, long)]
    pub ckb_address: String,
}

#[derive(Clap, Clone, Debug)]
pub struct LockEthArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub amount: u128,
    #[clap(short, long)]
    pub ckb_address: String,
}

#[derive(Clap, Clone, Debug)]
pub struct GenerateEthProofArgs {
    #[clap(short, long)]
    pub hash: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct MintArgs {}

#[derive(Clap, Clone, Debug)]
pub struct TransferFromCkbArgs {}

#[derive(Clap, Clone, Debug)]
pub struct BurnArgs {
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "https://localhost:8114")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct GenerateCkbProofArgs {
    #[clap(short, long)]
    pub tx_hash: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct UnlockArgs {}

#[derive(Clap, Clone, Debug)]
pub struct EthRelayArgs {}

#[derive(Clap, Clone, Debug)]
pub struct CkbRelayArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8116")]
    pub indexer_rpc_url: String,
}
