use clap::Clap;

#[derive(Clap, Clone, Debug)]
#[clap(version = "0.1", author = "LeonLi000 <matrix.skygirl@gmail.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    DevInit(DevInitArgs),
    TransferToCkb(TransferToCkbArgs),
    Approve(ApproveArgs),
    LockToken(LockTokenArgs),
    LockEth(LockEthArgs),
    GenerateEthProof(GenerateEthProofArgs),
    Mint(MintArgs),
    TransferFromCkb(TransferFromCkbArgs),
    TransferSudt(TransferSudtArgs),
    Burn(BurnArgs),
    GenerateCkbProof(GenerateCkbProofArgs),
    Unlock(UnlockArgs),
    QuerySudtBlance(SudtGetBalanceArgs),
    EthRelay(EthRelayArgs),
    CkbRelay(CkbRelayArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct DevInitArgs {
    #[clap(short = 'f', long)]
    pub force: bool,
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(short = 'k', long, default_value = "cli/privkeys/ckb_key")]
    pub private_key_path: String,
    #[clap(
        long,
        default_value = "../ckb-contracts/build/release/eth-bridge-typescript"
    )]
    pub bridge_typescript_path: String,
    #[clap(
        long,
        default_value = "../ckb-contracts/build/release/eth-bridge-lockscript"
    )]
    pub bridge_lockscript_path: String,
    #[clap(
        long,
        default_value = "../ckb-contracts/build/release/eth-light-client-typescript"
    )]
    pub light_client_typescript_path: String,
    #[clap(
        long,
        default_value = "../ckb-contracts/build/release/eth-recipient-typescript"
    )]
    pub recipient_typescript_path: String,
    #[clap(long, default_value = "cli/deps/simple_udt")]
    pub sudt_path: String,
    #[clap(long)]
    pub eth_contract_address: String,
    #[clap(long)]
    pub eth_token_address: String,
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
    #[clap(short = 'k', long, default_value = "cli/privkeys/eth_key")]
    pub private_key_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct LockTokenArgs {
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
    #[clap(short = 'k', long, default_value = "cli/privkeys/eth_key")]
    pub private_key_path: String,
    #[clap(long)]
    pub token: String,
    #[clap(short, long)]
    pub amount: u128,
    #[clap(short, long)]
    pub bridge_fee: u128,
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub sudt_extra_data: String,
    #[clap(long, default_value = "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37")]
    pub ckb_recipient_address: String,
}

#[derive(Clap, Clone, Debug)]
pub struct LockEthArgs {
    #[clap(short, long)]
    pub to: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
    #[clap(short = 'k', long, default_value = "cli/privkeys/eth_key")]
    pub private_key_path: String,
    #[clap(short, long)]
    pub amount: u128,
    #[clap(short, long)]
    pub bridge_fee: u128,
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub sudt_extra_data: String,
    #[clap(long, default_value = "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37")]
    pub ckb_recipient_address: String,
}

#[derive(Clap, Clone, Debug)]
pub struct GenerateEthProofArgs {
    #[clap(short, long)]
    pub hash: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct MintArgs {
    #[clap(short, long)]
    pub hash: String,
    #[clap(long, default_value = "http://127.0.0.1:9545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub indexer_url: String,
    #[clap(short = 'k', long, default_value = "cli/privkeys/ckb_key")]
    pub private_key_path: String,
    #[clap(short, long)]
    pub cell: String,
    #[clap(long)]
    pub eth_contract_address: String,
}

#[derive(Clap, Clone, Debug)]
pub struct TransferFromCkbArgs {}

#[derive(Clap, Clone, Debug)]
pub struct BurnArgs {
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8116")]
    pub indexer_rpc_url: String,
    #[clap(long)]
    pub token_addr: String,
    #[clap(long)]
    pub receive_addr: String,
    #[clap(long)]
    pub lock_contract_addr: String,
    #[clap(long)]
    pub burn_amount: u128,
    #[clap(long)]
    pub unlock_fee: u128,
}

#[derive(Clap, Clone, Debug)]
pub struct GenerateCkbProofArgs {
    #[clap(short, long)]
    pub tx_hash: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct UnlockArgs {
    #[clap(short, long)]
    pub from: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long)]
    pub tx_proof: String,
    #[clap(long)]
    pub tx_info: String,
    #[clap(long, default_value = "http://localhost:8545")]
    pub eth_rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct EthRelayArgs {
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8116")]
    pub indexer_rpc_url: String,
    #[clap(long, default_value = "/tmp/proof_data.json")]
    pub proof_data_path: String,
    /// cell typescript hex
    #[clap(short, long)]
    pub cell: String,
}

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

#[derive(Clap, Clone, Debug)]
pub struct TransferSudtArgs {
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8116")]
    pub indexer_rpc_url: String,
    #[clap(short, long)]
    pub to_addr: String,
    #[clap(long)]
    pub sudt_amount: u128,
    #[clap(long, default_value = "200")]
    pub ckb_amount: String,
    #[clap(long)]
    pub token_addr: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(long)]
    pub lock_contract_addr: String,
}

#[derive(Clap, Clone, Debug)]
pub struct SudtGetBalanceArgs {
    #[clap(long, default_value = "/tmp/.force-bridge-cli/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "http://localhost:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://localhost:8116")]
    pub indexer_rpc_url: String,
    #[clap(short, long)]
    pub addr: String,
    #[clap(long)]
    pub token_addr: String,
    #[clap(long)]
    pub lock_contract_addr: String,
}
