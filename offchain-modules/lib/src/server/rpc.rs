use super::{handlers::*, state::DappState};
use actix_web::{App, HttpServer};

pub async fn start(
    config_path: String,
    network: Option<String>,
    ckb_private_key_path: String,
    eth_private_key_path: String,
    listen_url: String,
    db_path: String,
) -> std::io::Result<()> {
    let dapp_state = DappState::new(
        config_path,
        network,
        ckb_private_key_path,
        eth_private_key_path,
        db_path,
    )
    .await
    .expect("init dapp server error");
    let local = tokio::task::LocalSet::new();
    let sys = actix_web::rt::System::run_in_tokio("server", &local);
    let _server_res = HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .data(dapp_state.clone())
            .service(index)
            .service(settings)
            .service(get_or_create_bridge_cell)
            .service(burn)
            .service(lock)
            .service(get_best_block_height)
            .service(get_sudt_balance)
            .service(get_eth_to_ckb_status)
            .service(relay_eth_to_ckb_proof)
            .service(get_crosschain_history)
    })
    .workers(18)
    .bind(&listen_url)?
    .run()
    .await?;
    sys.await?;
    Ok(())
}
