use super::types::ServerArgs;
use anyhow::Result;
use force_eth_lib::server::rpc::start;

pub fn server_handler(args: ServerArgs) -> Result<()> {
    start(
        args.config_path,
        args.ckb_rpc_url,
        args.indexer_url,
        args.private_key_path,
        args.listen_url,
        args.threads_num,
    );
    Ok(())
}
