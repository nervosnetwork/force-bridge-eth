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
use anyhow::{anyhow, bail, Result};
use force_sdk::util::ensure_indexer_sync;
use handlers::*;
use shellexpand::tilde;
use sqlx::mysql::MySqlPool;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

pub const REPLAY_RESIST_CELL_NUMBER: usize = 1000;
const REFRESH_RATE: usize = 100; // 100/100

// const REPLAY_RESIST_CELL_CAPACITY: &str = "315";

#[derive(Clone)]
pub struct DappState {
    pub config_path: String,
    pub network: Option<String>,
    pub deployed_contracts: DeployedContracts,
    pub init_token_privkey: String,
    pub refresh_cell_privkey: String,
    pub mint_privkey: String,
    pub create_bridge_cell_fee: String,
    pub indexer_url: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub db: MySqlPool,
    pub replay_resist_sender: mpsc::Sender<ReplayResistTask>,
    pub init_token_mutex: Arc<Mutex<i32>>,
}

pub struct ReplayResistTask {
    token: String,
    resp: oneshot::Sender<Result<String>>,
}

impl DappState {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        server_privkey_path: Vec<String>,
        mint_privkey_path: String,
        create_bridge_cell_fee: String,
        db_path: String,
        replay_resist_sender: mpsc::Sender<ReplayResistTask>,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let indexer_url = force_config.get_ckb_indexer_url(&network)?;
        if server_privkey_path.len() != 2 {
            bail!("invalid args: ckb private key path length should be 2");
        }
        let db = MySqlPool::connect(&db_path).await?;
        let init_token_mutex = Arc::new(Mutex::new(1));
        Ok(Self {
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            init_token_privkey: server_privkey_path[0].clone(),
            refresh_cell_privkey: server_privkey_path[1].clone(),
            mint_privkey: mint_privkey_path,
            create_bridge_cell_fee,
            deployed_contracts: force_config
                .deployed_contracts
                .expect("contracts should be deployed"),
            network,
            db,
            replay_resist_sender,
            init_token_mutex,
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
        privkey: String,
        is_create: bool,
    ) -> Result<Vec<String>> {
        to_ckb::get_or_create_bridge_cell(
            self.config_path.clone(),
            self.network.clone(),
            privkey,
            self.mint_privkey.clone(),
            self.create_bridge_cell_fee.clone(),
            token.to_string(),
            "".to_string(),
            0,
            true,
            cell_num,
            is_create,
        )
        .await
    }

    pub async fn try_refresh_replay_resist_cells(&self, token: &str) -> Result<()> {
        let fresh_cells = self
            .get_or_create_bridge_cell(
                token,
                REPLAY_RESIST_CELL_NUMBER,
                self.refresh_cell_privkey.clone(),
                false,
            )
            .await?;
        let (delete_cells, add_cells, available_cells_number) = self
            .prepare_cell_modification(fresh_cells, token.to_string())
            .await?;
        let (delete_cells, add_cells, _) = if add_cells.len() + available_cells_number
            < REPLAY_RESIST_CELL_NUMBER
        {
            log::warn!("need force create bridge cells: add_cells number {:?}, available_cells number {:?}", add_cells.len(), available_cells_number);
            let fresh_cells = self
                .get_or_create_bridge_cell(
                    token,
                    REPLAY_RESIST_CELL_NUMBER,
                    self.refresh_cell_privkey.clone(),
                    true,
                )
                .await?;
            self.prepare_cell_modification(fresh_cells, token.to_string())
                .await?
        } else {
            (delete_cells, add_cells, available_cells_number)
        };
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
        Ok(())
    }

    async fn prepare_cell_modification(
        &self,
        fresh_cells: Vec<String>,
        token: String,
    ) -> Result<(Vec<u64>, Vec<String>, usize)> {
        let available_cells = get_replay_resist_cells(&self.db, &token, "available").await?;
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
        Ok((delete_cells, add_cells, available_cells.len()))
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn start(
    config_path: String,
    network: Option<String>,
    server_private_key_path: Vec<String>,
    mint_private_key_path: String,
    lock_api_channel_bound: usize,
    create_bridge_cell_fee: String,
    listen_url: String,
    db_path: String,
) -> Result<()> {
    let (sender, mut receiver) = mpsc::channel(lock_api_channel_bound);
    let dapp_state = DappState::new(
        config_path,
        network,
        server_private_key_path,
        mint_private_key_path,
        create_bridge_cell_fee,
        db_path,
        sender.clone(),
    )
    .await?;
    let dapp_state_for_receiver = dapp_state.clone();

    tokio::spawn(async move {
        log::info!("start repaly resist cell channel receiver");
        let is_refreshing_replay_resist_cell = Arc::new(Mutex::new(false));
        while let Some(task) = receiver.recv().await {
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
                    .send(Err(anyhow!(
                        "replay resist cell is exhausted, please wait for create new cells"
                    )))
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
            if cell_count < REPLAY_RESIST_CELL_NUMBER * REFRESH_RATE / 100 {
                let mut is_refreshing = is_refreshing_replay_resist_cell.lock().await;
                if *is_refreshing {
                    log::info!("already refreshing replay resist cells: {:?}", &task.token);
                    continue;
                }
                *is_refreshing = true;
                log::info!("start refresh replay resist cells: {:?}", &task.token);
                let is_refreshing_replay_resist_cell = is_refreshing_replay_resist_cell.clone();
                let dapp_state_for_refresh = dapp_state_for_receiver.clone();
                let token = task.token.clone();
                tokio::spawn(async move {
                    let ret = dapp_state_for_refresh
                        .try_refresh_replay_resist_cells(&token)
                        .await;
                    let mut is_refreshing = is_refreshing_replay_resist_cell.lock().await;
                    *is_refreshing = false;
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
            .service(init_token)
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
    .workers(100)
    .bind(&listen_url)?
    .run()
    .await?;
    sys.await?;
    Ok(())
}
