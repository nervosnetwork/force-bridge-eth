use anyhow::Result;
use types::*;

use force_eth_lib::dapp::relayer::ckb_relayer::CkbTxRelay;

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
    let mut ckb_tx_relay = CkbTxRelay::new(args.config_path, args.network, args.db_args).await?;
    ckb_tx_relay.start().await?;
    Ok(())
}

async fn eth_tx_relay(_args: EthTxRelayerArgs) -> Result<()> {
    // TODO
    Ok(())
}
