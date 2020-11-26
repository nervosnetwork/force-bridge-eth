use super::{handlers::*, state::DappState};
use actix_web::{App, HttpServer};

pub async fn start(
    config_path: String,
    ckb_rpc_url: String,
    eth_rpc_url: String,
    indexer_url: String,
    private_key_path: String,
    listen_url: String,
) -> std::io::Result<()> {
    let dapp_state = DappState::new(
        config_path,
        indexer_url,
        ckb_rpc_url,
        eth_rpc_url,
        private_key_path,
    )
    .expect("init dapp server error");
    let local = tokio::task::LocalSet::new();
    let sys = actix_web::rt::System::run_in_tokio("server", &local);
    let _server_res = HttpServer::new(move || {
        App::new()
            .data(dapp_state.clone())
            .service(index)
            .service(settings)
            .service(get_or_create_bridge_cell)
            .service(burn)
            .service(lock)
            .service(get_best_block_height)
            .service(get_sudt_balance)
    })
    .bind(&listen_url)?
    .run()
    .await?;
    sys.await?;
    Ok(())
}
