use super::types::ServerArgs;
use anyhow::Result;
use force_eth_lib::server::rpc::start;

pub async fn server_handler(args: ServerArgs) -> Result<()> {
    Ok(start(
        args.config_path,
        args.ckb_rpc_url,
        args.eth_rpc_url,
        args.indexer_url,
        args.private_key_path,
        args.listen_url,
    )
    .await?)
}
