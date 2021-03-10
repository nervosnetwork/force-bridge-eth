use clap::Clap;

#[derive(Clap, Clone, Debug)]
pub enum DappCommand {
    Server(ServerArgs),
    ETHIndexer(EthIndexerArgs),
    CKBIndexer(CkbIndexerArgs),
    CkbTxRelayer(CkbTxRelayerArgs),
    EthTxRelayer(EthTxRelayerArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct ServerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'c', long)]
    pub server_private_key_path: Vec<String>,
    #[clap(long)]
    pub mint_private_key_path: String,
    #[clap(long, default_value = "5000")]
    pub lock_api_channel_bound: usize,
    #[clap(long, default_value = "0.9")]
    pub create_bridge_cell_fee: String,
    #[clap(short, long, default_value = "127.0.0.1:3030")]
    pub listen_url: String,
    #[clap(long, default_value = "mysql://root:@127.0.0.1:3306/serverdb")]
    pub db_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct EthIndexerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(long, default_value = "mysql://root:@127.0.0.1:3306/forcedb")]
    pub db_path: String,
    #[clap(
        long,
        default_value = "9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"
    )]
    pub recipient_lockscript_code_hash: String,
}

#[derive(Clap, Clone, Debug)]
pub struct CkbIndexerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "mysql://root:@127.0.0.1:3306/forcedb")]
    pub db_path: String,
    #[clap(long)]
    pub network: Option<String>,
}

#[derive(Clap, Clone, Debug)]
pub struct CkbTxRelayerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'k', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "mysql://root:@127.0.0.1:3306/forcedb")]
    pub db_path: String,
    #[clap(long, default_value = "~/.force-bridge/ckb-rocksdb")]
    pub rocksdb_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct EthTxRelayerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(short = 'p', long)]
    pub private_key_path: String,
    #[clap(long, default_value = "100")]
    pub mint_concurrency: u64,
    #[clap(long, default_value = "1000")]
    pub minimum_cell_capacity: u64,
    #[clap(long, default_value = "mysql://root:@127.0.0.1:3306/forcedb")]
    pub db_path: String,
}
