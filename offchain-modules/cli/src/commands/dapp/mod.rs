use anyhow::Result;
use force_eth_lib::dapp::EthTxRelayer;
use types::*;

pub mod types;

pub async fn dapp_handle(command: DappCommand) -> Result<()> {
    match command {
        DappCommand::Server(args) => server(args).await,
        DappCommand::Indexer(args) => indexer(args).await,
        DappCommand::CkbTxRelayer(args) => ckb_tx_relay(args).await,
        DappCommand::EthTxRelayer(args) => eth_tx_relay(args).await,
    }
}

async fn server(_args: ServerArgs) -> Result<()> {
    // TODO
    Ok(())
}

async fn indexer(_args: IndexerArgs) -> Result<()> {
    // TODO
    Ok(())
}

async fn ckb_tx_relay(_args: CkbTxRelayerArgs) -> Result<()> {
    // TODO
    Ok(())
}

async fn eth_tx_relay(args: EthTxRelayerArgs) -> Result<()> {
    let eth_tx_relayer = EthTxRelayer::new(
        args.config_path,
        args.network,
        args.private_key_path,
        args.db_args,
    )?;
    eth_tx_relayer.start().await
}
