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
}

#[derive(Clap, Clone, Debug)]
pub struct ServerArgs {
    #[clap(long, default_value = "conf/config.toml")]
    pub config_path: String,
    #[clap(long, default_value = "conf/ckb_key")]
    pub ckb_key_path: String,
    #[clap(long, default_value = "conf/eth_key")]
    pub eth_key_path: String,
    #[clap(
        long,
        default_value = "590000001000000030000000310000001313a0eaa571a9168e44ceba1a0d0a328840d9de43aab2388af7c860b57c9a0c01240000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f8107000000"
    )]
    pub cell_script: String,
    #[clap(long, default_value = "0.0.0.0:3031")]
    pub listen_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8114")]
    pub ckb_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub ckb_indexer_url: String,
}
