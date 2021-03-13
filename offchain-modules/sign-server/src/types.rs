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
    // Indexer(IndexerArgs),
}

#[derive(Clap, Clone, Debug)]
pub struct ServerArgs {
    #[clap(long, default_value = "~/.sign_server/rocksdb")]
    pub db_path: String,
    #[clap(short = 'c', default_value = "~/.sign_server/ckb_key")]
    pub ckb_private_key_path: String,
    #[clap(short = 'e', default_value = "~/.sign_server/eth_key")]
    pub eth_private_key_path: String,
    #[clap(
        default_value = "59000000100000003000000031000000c24ed13d860852875f427a3fac56bc955ad3b83d06b33b12320d0378637c03e000240000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f8107000000"
    )]
    pub cell_script: String,
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub eth_rpc_url: String,
    #[clap(long, default_value = "http://127.0.0.1:8116")]
    pub ckb_indexer_url: String,
}

// #[derive(Clap, Clone, Debug)]
// pub struct IndexerArgs {
//     #[clap(
//         default_value = "59000000100000003000000031000000c24ed13d860852875f427a3fac56bc955ad3b83d06b33b12320d0378637c03e000240000005edca2d744b6eaa347de7ff0edcd2e6e88ab8f2836bcbd0df0940026956e5f8107000000"
//     )]
//     pub cell_script: String,
//     #[clap(long, default_value = "http://127.0.0.1:8545")]
//     pub eth_rpc_url: String,
//     #[clap(long, default_value = "http://127.0.0.1:8116")]
//     pub ckb_indexer_url: String,
// }
