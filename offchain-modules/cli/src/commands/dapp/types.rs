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
    pub ckb_private_key_path: String,
    #[clap(short = 'e', long)]
    pub eth_private_key_path: String,
    #[clap(short, long, default_value = "127.0.0.1:3030")]
    pub listen_url: String,
    #[clap(long, default_value = "~/.force-bridge/force.db")]
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
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub ckb_indexer_url: String,
}

#[derive(Clap, Clone, Debug)]
pub struct CkbIndexerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "mysql://root:@127.0.0.1:3306/forcedb")]
    pub db_path: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub ckb_indexer_url: String,
    #[clap(long)]
    pub network: Option<String>,
}

#[derive(Clap, Clone, Debug)]
pub struct CkbTxRelayerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
}

#[derive(Clap, Clone, Debug)]
pub struct EthTxRelayerArgs {
    #[clap(long, default_value = "~/.force-bridge/config.toml")]
    pub config_path: String,
    #[clap(long)]
    pub network: Option<String>,
}
