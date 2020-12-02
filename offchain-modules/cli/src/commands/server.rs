use super::types::ServerArgs;
use anyhow::Result;
use force_eth_lib::server::rpc::start;

pub async fn server_handler(args: ServerArgs) -> Result<()> {
    Ok(start(
        args.config_path,
        args.network,
        args.ckb_private_key_path,
        args.eth_private_key_path,
        args.listen_url,
        args.db_path,
    )
    .await?)
}
