use clap::Clap;

#[derive(Clap, Clone, Debug)]
#[clap(version = "0.1", author = "LeonLi000 <matrix.skygirl@gmail.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    Server(ServerArgs),
    InitCkbLightContract(InitCkbLightContractArgs),
    Init(InitArgs),
    DeployCKB(DeployCKBArgs),
    CreateBridgeCell(CreateBridgeCellArgs),
    TransferToCkb(TransferToCkbArgs),
    Approve(ApproveArgs),
    LockToken(LockTokenArgs),
    LockEth(LockEthArgs),
    // GenerateEthProof(GenerateEthProofArgs),
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
pub struct ServerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'c', long)]
    pub ckb_private_key_path: String,
    #[clap(short = 'e', long)]
    pub eth_private_key_path: String,
    #[clap(short, long, default_value = "127.0.0.1:3030")]
    pub listen_url: String,
    #[clap(long, default_value = "~/.force-bridge/force.db")]
    pub db_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct CreateBridgeCellArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long)]
    pub eth_token_address: String,
    #[clap(long)]
    pub recipient_address: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(long, default_value = "283")]
    pub capacity: String,
    #[clap(long, default_value = "0")]
    pub bridge_fee: u128,
}

#[derive(Clap, Clone, Debug)]
pub struct InitCkbLightContractArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub init_height: Option<u64>,
    #[clap(short, long)]
    pub finalized_gc: u64,
    #[clap(short, long)]
    pub canonical_gc: u64,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub wait: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct InitArgs {
    #[clap(short = 'f', long)]
    pub force: bool,
    #[clap(short = 'p', long)]
    pub project_path: String,
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "docker-dev-chain")]
    pub default_network: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub ckb_indexer_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub ethereum_rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct DeployCKBArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(long)]
    pub eth_dag_path: Option<String>,
}

#[derive(Clap, Clone, Debug)]
pub struct TransferToCkbArgs {}

#[derive(Clap, Clone, Debug)]
pub struct ApproveArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub erc20_addr: String,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub wait: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct LockTokenArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long)]
    pub token: String,
    #[clap(short, long)]
    pub amount: u128,
    #[clap(short, long)]
    pub bridge_fee: u128,
    #[clap(long)]
    pub sudt_extra_data: String,
    #[clap(long)]
    pub ckb_recipient_address: String,
    #[clap(long)]
    pub replay_resist_outpoint: String,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub wait: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct LockEthArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub amount: u128,
    #[clap(short, long)]
    pub bridge_fee: u128,
    #[clap(long)]
    pub sudt_extra_data: Option<String>,
    #[clap(long)]
    pub ckb_recipient_address: String,
    #[clap(long)]
    pub replay_resist_outpoint: String,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub wait: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct GenerateEthProofArgs {
    #[clap(short, long)]
    pub hash: String,
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub rpc_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct MintArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub hash: String,
}

#[derive(Clap, Clone, Debug)]
pub struct TransferFromCkbArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(long)]
    pub eth_privkey_path: String,
    #[clap(long)]
    pub ckb_privkey_path: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(long)]
    pub token_addr: String,
    #[clap(long)]
    pub receive_addr: String,
    #[clap(long)]
    pub burn_amount: u128,
    #[clap(long)]
    pub unlock_fee: u128,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub wait: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct BurnArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(long)]
    pub token_addr: String,
    #[clap(long)]
    pub receive_addr: String,
    #[clap(long)]
    pub burn_amount: u128,
    #[clap(long)]
    pub unlock_fee: u128,
}

#[derive(Clap, Clone, Debug)]
pub struct GenerateCkbProofArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short, long)]
    pub tx_hash: String,
}

#[derive(Clap, Clone, Debug)]
pub struct UnlockArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub to: String,
    #[clap(long)]
    pub tx_proof: String,
    #[clap(long)]
    pub tx_info: String,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub wait: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct EthRelayArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "data/proof_data.json")]
    pub proof_data_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct CkbRelayArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub per_amount: u64,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
}

#[derive(Clap, Clone, Debug)]
pub struct TransferSudtArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
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
}

#[derive(Clap, Clone, Debug)]
pub struct SudtGetBalanceArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short, long)]
    pub addr: String,
    #[clap(long)]
    pub token_addr: String,
}
