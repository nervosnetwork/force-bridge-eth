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
    Indexer(IndexerArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct ServerArgs {
    #[clap(long, default_value = "~/.sign_server/rocksdb")]
    pub db_path: String,
    #[clap(short = 'c', default_value = "~/.sign_server/keys/ckb_key")]
    pub ckb_private_key_path: String,
    #[clap(short = 'e', default_value = "~/.sign_server/keys/eth_key")]
    pub eth_private_key_path: String,
}

#[derive(Clap, Clone, Debug)]
pub struct IndexerArgs {
    #[clap(default_value = "xxx")]
    pub cell_script: String,
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub ckb_indexer_url: String,
}
