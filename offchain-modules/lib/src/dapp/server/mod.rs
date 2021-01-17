pub mod error;
pub mod handlers;
pub mod types;

use crate::util::ckb_tx_generator::Generator;
use crate::util::config::{DeployedContracts, ForceConfig};
use crate::util::eth_util::Web3Client;
use crate::transfer::to_ckb;
use super::db::server::use_replay_resist_cell;
use anyhow::{anyhow, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use force_sdk::util::ensure_indexer_sync;
use shellexpand::tilde;
use sqlx::mysql::MySqlPool;
use std::collections::hash_set::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use handlers::*;
use actix_web::{App, HttpServer};

#[derive(Clone)]
pub struct DappState {
    pub config_path: String,
    pub network: Option<String>,
    pub deployed_contracts: DeployedContracts,
    pub ckb_privkey_path: String,
    pub indexer_url: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub db: MySqlPool,
    pub replay_resist_sender: mpsc::Sender<ReplayResistTask>
}

pub struct ReplayResistTask {
    token: String,
    resp: oneshot::Sender<String>
}

impl DappState {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        ckb_privkey_path: String,
        db_path: String,
        replay_resist_sender: mpsc::Sender<ReplayResistTask>
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let indexer_url = force_config.get_ckb_indexer_url(&network)?;

        let db = MySqlPool::connect(&db_path).await?;
        Ok(Self {
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            ckb_privkey_path,
            deployed_contracts: force_config
                .deployed_contracts
                .expect("contracts should be deployed"),
            network,
            db,
            replay_resist_sender
        })
    }

    pub async fn get_generator(&self) -> Result<Generator> {
        let mut generator = Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.deployed_contracts.clone(),
        )
            .map_err(|e| anyhow!("new geneartor fail, err: {}", e))?;
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
            .await
            .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
        Ok(generator)
    }

    pub fn get_web3_client(&self) -> Web3Client {
        Web3Client::new(self.eth_rpc_url.clone())
    }

    pub async fn refresh_replay_resist_cells(&self, token: String) -> Result<String> {
        let onchain_cells = to_ckb::get_or_create_bridge_cell(
            self.config_path.clone(),
            self.network.clone(),
            self.ckb_privkey_path.clone(),
            "0.9".to_string(),
            "315".to_string(),
            token,
            "".to_string(),
            0,
            true,
            100,
        )
            .await?;
        Ok(onchain_cells[0].clone())
    }
}

pub async fn start(
    config_path: String,
    network: Option<String>,
    ckb_private_key_path: String,
    listen_url: String,
    db_path: String,
) -> std::io::Result<()> {
    let (sender, mut receiver) = mpsc::channel(1000);
    let dapp_state = DappState::new(
        config_path,
        network,
        ckb_private_key_path,
        db_path,
        sender.clone()
    )
        .await
        .expect("init dapp server error");
    let dapp_state_2 = dapp_state.clone();
    let local = tokio::task::LocalSet::new();
    let sys = actix_web::rt::System::run_in_tokio("server", &local);
    let _server_res = HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .data(dapp_state.clone())
            .service(index)
            .service(settings)
            .service(burn)
            .service(lock)
            .service(get_best_block_height)
            .service(get_sudt_balance)
            .service(get_eth_to_ckb_status)
            .service(get_crosschain_history)
    })
        .workers(80)
        .bind(&listen_url)?
        .run()
        .await?;

    tokio::spawn(async move {
        while let Some(task) = receiver.recv().await {
            let (cell_count, replay_resist_cell) = use_replay_resist_cell(&dapp_state_2.db, &task.token).await.expect("connect db succeed");
            if cell_count < 100 {

            }
            if replay_resist_cell == "" {
            }
            task.resp.send(replay_resist_cell);
        }
    });
    sys.await?;
    Ok(())
}
