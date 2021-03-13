use super::dapp::types::DappCommand;
use clap::Clap;

#[derive(Clap, Clone, Debug)]
#[clap(version = "0.1", author = "LeonLi000 <matrix.skygirl@gmail.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Clone, Debug)]
pub enum SubCommand {
    InitCkbLightContract(InitCkbLightContractArgs),
    InitConfig(InitConfigArgs),
    InitMultiSignAddress(InitMultiSignAddressArgs),
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
    Transfer(TransferArgs),
    Burn(BurnArgs),
    GenerateCkbProof(GenerateCkbProofArgs),
    Unlock(UnlockArgs),
    QuerySudtBlance(SudtGetBalanceArgs),
    EthRelay(EthRelayArgs),
    CkbRelay(CkbRelayArgs),
    RelayerMonitor(RelayerMonitorArgs),
    RecycleBridgeCell(RecycleBridgeCellArgs),
    Dapp(DappCommand),
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
    #[clap(long, default_value = "1")]
    pub number: usize,
    #[clap(long, default_value = "0")]
    pub bridge_fee: u128,
    #[clap(short = 's', long)]
    pub simple_typescript: bool,
    #[clap(long)]
    pub force_create: bool,
}

#[derive(Clap, Clone, Debug)]
pub struct RecycleBridgeCellArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "0.1")]
    pub tx_fee: String,
    #[clap(long)]
    pub outpoints: Option<Vec<String>>,
    #[clap(long, default_value = "5000")]
    pub max_recycle_count: usize,
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
pub struct InitConfigArgs {
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
    #[clap(long, default_value = "~/.force-bridge/eth-rocksdb")]
    pub eth_rocksdb_path: String,
    #[clap(long, default_value = "~/.force-bridge/ckb-rocksdb")]
    pub ckb_rocksdb_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct InitMultiSignAddressArgs {
    #[clap(long)]
    pub multi_address: Vec<String>,
    #[clap(long, default_value = "2")]
    pub threshold: u8,
    #[clap(long, default_value = "0")]
    pub require_first_n: u8,
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long)]
    pub network: Option<String>,
}

#[derive(Clap, Clone, Debug)]
pub struct DeployCKBArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long)]
    pub type_id: bool,
    #[clap(long)]
    pub sudt: bool,
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
    pub proof: String,
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
    #[clap(long)]
    pub multisig_privkeys: Vec<String>,
    #[clap(long, default_value = "15")]
    pub confirm: u64,
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
    #[clap(long, default_value = "50")]
    pub max_tx_count: u64,
    #[clap(short, long, default_value = "0")]
    pub gas_price: u64,
    #[clap(long)]
    pub mutlisig_privkeys: Vec<String>,
    #[clap(long, default_value = "15")]
    pub confirm: u64,
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
pub struct TransferArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(short, long)]
    pub to_addr: String,
    #[clap(long)]
    pub ckb_amount: String,
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

#[derive(Clap, Clone, Debug)]
pub struct RelayerMonitorArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(long, default_value = "100")]
    pub ckb_alarm_number: u64,
    #[clap(long, default_value = "100")]
    pub eth_alarm_number: u64,
    #[clap(long)]
    pub eth_header_conservator: Option<Vec<String>>,
    #[clap(long)]
    pub ckb_header_conservator: Option<Vec<String>>,
    #[clap(long)]
    pub eth_indexer_conservator: Option<Vec<String>>,
    #[clap(long)]
    pub ckb_indexer_conservator: Option<Vec<String>>,
    #[clap(long)]
    pub alarm_url: String,
    #[clap(long, default_value = "5")]
    pub minute_interval: u64,
    #[clap(long)]
    pub db_path: Option<String>,
    #[clap(long, default_value = "all")]
    pub mode: String,
    #[clap(long, default_value = "100")]
    pub ckb_alarm_balance: u64,
    #[clap(long, default_value = "100")]
    pub eth_alarm_balance: u64,
    #[clap(long)]
    pub eth_balance_conservator: String,
    #[clap(long)]
    pub ckb_balance_conservator: String,
}
