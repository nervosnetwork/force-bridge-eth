use anyhow::Result;
use force_eth_lib::dapp::EthTxRelayer;
use force_eth_lib::dapp::indexer::ckb_indexer::CkbIndexer;
use force_eth_lib::dapp::indexer::eth_indexer::EthIndexer;
use force_eth_lib::dapp::relayer::ckb_relayer::CkbTxRelay;
use types::*;

pub mod types;

pub async fn dapp_handle(command: DappCommand) -> Result<()> {
    match command {
        DappCommand::Server(args) => server(args).await,
        DappCommand::ETHIndexer(args) => eth_indexer(args).await,
        DappCommand::CKBIndexer(args) => ckb_indexer(args).await,
        DappCommand::CkbTxRelayer(args) => ckb_tx_relay(args).await,
        DappCommand::EthTxRelayer(args) => eth_tx_relay(args).await,
    }
}

async fn server(_args: ServerArgs) -> Result<()> {
    // TODO
    Ok(())
}

async fn eth_indexer(args: EthIndexerArgs) -> Result<()> {
    let mut eth_indexer = EthIndexer::new(args.config_path, args.network, args.db_path).await?;
    loop {
        let res = eth_indexer.start().await;
        if let Err(err) = res {
            log::error!("An error occurred during the eth_indexer. Err: {:?}", err)
        }
        tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
    }
}

async fn ckb_indexer(args: CkbIndexerArgs) -> Result<()> {
    let mut ckb_indexer = CkbIndexer::new(
        args.config_path,
        args.db_path,
        args.ckb_rpc_url,
        args.ckb_indexer_url,
    )
    .await?;
    loop {
        let res = ckb_indexer.start().await;
        if let Err(err) = res {
            log::error!("An error occurred during the ckb_indexer. Err: {:?}", err)
        }
        tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
    }
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

async fn eth_tx_relay(args: EthTxRelayerArgs) -> Result<()> {
    let eth_tx_relayer = EthTxRelayer::new(
        args.config_path,
        args.network,
        args.private_key_path,
        args.mint_concurrency,
        args.minimum_cell_capacity,
        args.db_url,
    )
    .await?;
    eth_tx_relayer.start().await
}
