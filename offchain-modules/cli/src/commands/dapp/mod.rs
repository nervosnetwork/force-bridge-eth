use anyhow::Result;
use force_eth_lib::dapp::CkbTxRelay;
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

async fn ckb_tx_relay(args: CkbTxRelayerArgs) -> Result<()> {
    let mut ckb_tx_relay = CkbTxRelay::new(
        args.config_path,
        args.network,
        args.db_args,
        args.private_key_path,
    )
    .await?;
    ckb_tx_relay.start().await
}

async fn eth_tx_relay(_args: EthTxRelayerArgs) -> Result<()> {
    // TODO
    Ok(())
}
