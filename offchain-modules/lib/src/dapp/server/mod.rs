pub mod errors;
pub mod handlers;
pub mod types;

use super::db::server::{
    add_replay_resist_cells, delete_replay_resist_cells, get_replay_resist_cells,
    use_replay_resist_cell,
};
use crate::transfer::to_ckb;
use crate::util::ckb_tx_generator::Generator;
use crate::util::config::{DeployedContracts, ForceConfig};
use crate::util::eth_util::Web3Client;
use actix_web::{App, HttpServer};
use anyhow::{anyhow, Result};
use force_sdk::util::ensure_indexer_sync;
use handlers::*;
use shellexpand::tilde;
use sqlx::mysql::MySqlPool;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

const REPLAY_RESIST_CHANNEL_BOUND: usize = 5000;
pub const REPLAY_RESIST_CELL_NUMBER: usize = 1000;
const REFRESH_RATE: usize = 50; // 50/100
const REPLAY_RESIST_CELL_CAPACITY: &str = "315";
const CREATE_REPLAY_RESIST_CELL_FEE: &str = "0.9";

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
    pub replay_resist_sender: mpsc::Sender<ReplayResistTask>,
}

pub struct ReplayResistTask {
    token: String,
    resp: oneshot::Sender<Result<String>>,
}

impl DappState {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        ckb_privkey_path: String,
        db_path: String,
        replay_resist_sender: mpsc::Sender<ReplayResistTask>,
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
            replay_resist_sender,
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

    pub async fn get_or_create_bridge_cell(
        &self,
        token: &str,
        cell_num: usize,
    ) -> Result<Vec<String>> {
        to_ckb::get_or_create_bridge_cell(
            self.config_path.clone(),
            self.network.clone(),
            self.ckb_privkey_path.clone(),
            CREATE_REPLAY_RESIST_CELL_FEE.to_string(),
            REPLAY_RESIST_CELL_CAPACITY.to_string(),
            token.to_string(),
            "".to_string(),
            0,
            true,
            cell_num,
        )
        .await
    }

    pub async fn refresh_replay_resist_cells(
        &self,
        token: &str,
        is_refreshing: Arc<Mutex<bool>>,
    ) -> Result<()> {
        let fresh_cells = self
            .get_or_create_bridge_cell(token, REPLAY_RESIST_CELL_NUMBER * 2)
            .await?;
        let mut is_refreshing = is_refreshing.lock().await;
        let (delete_cells, add_cells) = self
            .prepare_cell_modification(fresh_cells, token.to_string())
            .await?;
        add_replay_resist_cells(&self.db, &add_cells, &token).await?;
        log::info!(
            "refresh cells, token {:?}, add number: {:?}",
            token,
            add_cells.len()
        );
        delete_replay_resist_cells(&self.db, &delete_cells).await?;
        log::info!(
            "refresh cells, token {:?}, delete number: {:?}",
            token,
            delete_cells.len()
        );
        *is_refreshing = false;
        Ok(())
    }

    async fn prepare_cell_modification(
        &self,
        fresh_cells: Vec<String>,
        token: String,
    ) -> Result<(Vec<u64>, Vec<String>)> {
        let used_cells = get_replay_resist_cells(&self.db, &token, "used").await?;
        let used_cell_outpoints: Vec<String> = used_cells
            .iter()
            .map(|cell| cell.outpoint.clone())
            .collect();
        let delete_cells = used_cells
            .into_iter()
            .filter(|cell| !fresh_cells.contains(&cell.outpoint))
            .map(|cell| cell.id)
            .collect();
        let available_cells = get_replay_resist_cells(&self.db, &token, "available").await?;
        let available_cells_outpoints: Vec<String> = available_cells
            .iter()
            .map(|cell| cell.outpoint.clone())
            .collect();
        let add_cells: Vec<String> = fresh_cells
            .into_iter()
            .filter(|cell| {
                !used_cell_outpoints.contains(cell) && !available_cells_outpoints.contains(cell)
            })
            .collect();
        Ok((delete_cells, add_cells))
    }
}

pub async fn start(
    config_path: String,
    network: Option<String>,
    ckb_private_key_path: String,
    listen_url: String,
    db_path: String,
) -> std::io::Result<()> {
    let (sender, mut receiver) = mpsc::channel(REPLAY_RESIST_CHANNEL_BOUND);
    let dapp_state = DappState::new(
        config_path,
        network,
        ckb_private_key_path,
        db_path,
        sender.clone(),
    )
    .await
    .expect("init dapp server succeed");
    let dapp_state_for_receiver = dapp_state.clone();

    tokio::spawn(async move {
        log::info!("start repaly resist cell channel receiver");
        let is_refreshing_replay_resist_cell = Arc::new(Mutex::new(false));
        while let Some(task) = receiver.recv().await {
            let mut is_refreshing = is_refreshing_replay_resist_cell.lock().await;
            let result = use_replay_resist_cell(&dapp_state_for_receiver.db, &task.token).await;
            if let Err(e) = result {
                log::error!("use replay resist cell error: {:?}", e);
                task.resp
                    .send(Err(e))
                    .expect("send response to lock handler succeed");
                continue;
            }
            let (cell_count, replay_resist_cell) = result.unwrap();
            if replay_resist_cell == "" {
                task.resp
                    .send(Err(anyhow!("replay resist cell is exhausted, please wait")))
                    .expect("send response to lock handler succeed");
            } else {
                task.resp
                    .send(Ok(replay_resist_cell))
                    .expect("send response to lock handler succeed");
            }

            log::info!(
                "remaining replay resist cells count: {:?} {:?}",
                &task.token,
                cell_count
            );
            if cell_count < REPLAY_RESIST_CELL_NUMBER * REFRESH_RATE / 100 && !(*is_refreshing) {
                log::info!("start refresh replay resist cells: {:?}", &task.token);
                *is_refreshing = true;
                let is_refreshing_replay_resist_cell = is_refreshing_replay_resist_cell.clone();
                let dapp_state_for_refresh = dapp_state_for_receiver.clone();
                let token = task.token.clone();
                tokio::spawn(async move {
                    let ret = dapp_state_for_refresh
                        .refresh_replay_resist_cells(&token, is_refreshing_replay_resist_cell)
                        .await;
                    if ret.is_err() {
                        log::error!(
                            "refresh replay resist cells error: {:?} {:?}",
                            &token,
                            ret.unwrap_err()
                        );
                    } else {
                        log::info!("refresh replay resist cells succeed: {:?}", &token);
                    }
                });
            }
        }
    });

    let local = tokio::task::LocalSet::new();
    let sys = actix_web::rt::System::run_in_tokio("server", &local);
    let _server_res = HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .data(dapp_state.clone())
            .service(lock)
            .service(burn)
            .service(get_eth_to_ckb_status)
            .service(get_ckb_to_eth_status)
            .service(get_crosschain_history)
            .service(get_sudt_balance)
            .service(get_best_block_height)
            .service(settings)
            .service(index)
    })
    .workers(80)
    .bind(&listen_url)?
    .run()
    .await?;
    sys.await?;
    Ok(())
}
