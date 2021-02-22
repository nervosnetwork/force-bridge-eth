use crate::dapp::db::eth_relayer::{
    delete_relayed_tx, get_mint_tasks, get_retry_tasks, last_relayed_number, latest_index_number,
    store_mint_tasks, update_relayed_tx, MintTask,
};
use crate::transfer::to_ckb::send_eth_spv_proof_tx;
use crate::util::ckb_tx_generator::{Generator, CONFIRM};
use crate::util::ckb_util::{get_eth_client_tip_number, parse_privkey_path, ETHSPVProofJson};
use crate::util::config::ForceConfig;
use anyhow::{anyhow, bail, Result};
use ckb_sdk::constants::ONE_CKB;
use ckb_sdk::AddressPayload;
use ckb_sdk::{HumanCapacity, SECP256K1};
use ckb_types::core::Capacity;
use ckb_types::packed::{CellOutput, OutPoint, Script};
use ckb_types::prelude::Pack;
use ckb_types::H256;
use force_sdk::cell_collector::get_capacity_cells_for_mint;
use force_sdk::tx_helper::TxHelper;
use force_sdk::util::ensure_indexer_sync;
use futures::future::join_all;
use molecule::prelude::{Builder, Entity};
use secp256k1::SecretKey;
use shellexpand::tilde;
use sqlx::MySqlPool;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct EthTxRelayer {
    pub config_path: String,
    pub force_config: ForceConfig,
    pub network: Option<String>,
    pub ckb_rpc_url: String,
    pub ckb_indexer_url: String,
    pub private_key: SecretKey,
    pub db_pool: MySqlPool,
    pub mint_concurrency: u64,
    pub minimum_cell_capacity: u64,
}

impl EthTxRelayer {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        private_key: String,
        mint_concurrency: u64,
        minimum_cell_capacity: u64,
        db_url: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let private_key = parse_privkey_path(private_key.as_str(), &force_config, &network)?;
        let db_pool = MySqlPool::connect(db_url.as_str()).await?;
        Ok(EthTxRelayer {
            config_path,
            force_config,
            network,
            ckb_rpc_url,
            ckb_indexer_url,
            private_key,
            db_pool,
            mint_concurrency,
            minimum_cell_capacity: minimum_cell_capacity * ONE_CKB,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut last_relayed_number = last_relayed_number(&self.db_pool).await?;
        loop {
            let res = self.relay(last_relayed_number).await;
            if let Err(e) = res {
                log::error!("encountered an error when relay eth tx: {:?}", e);
            } else {
                last_relayed_number = res.unwrap();
            }
            tokio::time::delay_for(Duration::from_secs(15)).await
        }
    }

    async fn relay(&self, last_relayed_number: u64) -> Result<u64> {
        let capacity_cells = self.capacity_cells_for_mint().await?;
        if capacity_cells.is_empty() {
            log::info!("generate capacity cells for mint success, jump to next relay round");
            return Ok(last_relayed_number);
        }
        let client_confirmed_number = self.client_confirmed_number().await?;
        let latest_index_number = latest_index_number(&self.db_pool).await?;
        log::info!(
            "start new relay round: last relayed number: {}, client confirmed number: {}, latest indexed number: {}",
            last_relayed_number,
            client_confirmed_number,
            latest_index_number
        );
        let relay_to_number = std::cmp::min(client_confirmed_number, latest_index_number);
        let retry_tasks = get_retry_tasks(&self.db_pool).await?;
        log::info!("total retry tasks: {}", retry_tasks.len());
        let mut mint_tasks =
            get_mint_tasks(&self.db_pool, last_relayed_number, relay_to_number).await?;
        log::info!("total mint tasks: {}", mint_tasks.len());
        store_mint_tasks(&self.db_pool, &mint_tasks).await?;
        mint_tasks.extend(retry_tasks);

        let mint_count = std::cmp::min(mint_tasks.len(), capacity_cells.len());
        let mut mint_futures = vec![];
        for i in 0..mint_count {
            mint_futures.push(self.mint(&mint_tasks[i], &capacity_cells[i]));
        }
        if !mint_futures.is_empty() {
            log::info!("start send {} mint txs", mint_count);
            let now = Instant::now();
            let res_all = join_all(mint_futures).await;
            for res in res_all.iter() {
                if let Err(error) = res {
                    log::error!("mint error : {:?}", error);
                }
            }
            log::info!("mint {} txs elapsed {:?}", mint_count, now.elapsed());
        }
        Ok(relay_to_number)
    }

    async fn mint(&self, task: &MintTask, capacity_cell: &OutPoint) -> Result<()> {
        if let Err(error) = self.try_mint(&task, capacity_cell).await {
            if error.to_string().contains("irreparable error") {
                update_relayed_tx(
                    &self.db_pool,
                    task.lock_tx_hash.clone(),
                    "irreparable error".to_string(),
                    error.to_string(),
                )
                .await?;
                log::error!(
                    "mint for lock tx {:?} failed with irreparable error: {:?}",
                    task.lock_tx_hash,
                    error
                );
            } else {
                update_relayed_tx(
                    &self.db_pool,
                    task.lock_tx_hash.clone(),
                    "retryable".to_string(),
                    error.to_string(),
                )
                .await?;
                log::error!(
                    "mint for lock tx {:?} failed with retryable error: {:?}",
                    task.lock_tx_hash,
                    error
                );
            }
        } else {
            delete_relayed_tx(&self.db_pool, task.lock_tx_hash.clone()).await?;
            log::info!("mint for lock tx {:?} succeed", task.lock_tx_hash);
        }
        Ok(())
    }

    async fn try_mint(&self, task: &MintTask, capacity_cell: &OutPoint) -> Result<H256> {
        let mut generator = self.get_generator().await?;
        let lock_tx_proof: ETHSPVProofJson = serde_json::from_str(task.lock_tx_proof.as_str())?;
        send_eth_spv_proof_tx(
            &mut generator,
            self.config_path.clone(),
            task.lock_tx_hash.clone(),
            &lock_tx_proof,
            self.private_key,
            Some(capacity_cell.clone()),
        )
        .await
    }

    async fn capacity_cells_for_mint(&self) -> Result<Vec<OutPoint>> {
        let mut generator = self.get_generator().await?;
        let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &self.private_key);
        let address_payload = AddressPayload::from_pubkey(&from_public_key);
        let from_lockscript = Script::from(&address_payload);
        let capacity_cells = get_capacity_cells_for_mint(
            &mut generator.indexer_client,
            from_lockscript.clone(),
            self.minimum_cell_capacity,
            self.mint_concurrency,
        )
        .map_err(|e| {
            anyhow!(
                "get live capacity cells from indexer error when mint: {:?}",
                e
            )
        })?;
        if (capacity_cells.len() as u64) < self.mint_concurrency {
            log::info!("capacity cells for this round not enough");
            self.generate_capacity_cells(&mut generator, from_lockscript.clone())
                .await?;
            Ok(vec![])
        } else {
            let ret = capacity_cells
                .into_iter()
                .map(|cell| cell.out_point.into())
                .collect();
            Ok(ret)
        }
    }

    async fn generate_capacity_cells(
        &self,
        generator: &mut Generator,
        lockscript: Script,
    ) -> Result<()> {
        let tx_fee: u64 = HumanCapacity::from_str("0.9")
            .map_err(|e| anyhow!(e))?
            .into();
        let mut tx_helper = TxHelper::default();
        for _ in 0..self.mint_concurrency {
            let cell_output = CellOutput::new_builder()
                .capacity(Capacity::shannons(10 * self.minimum_cell_capacity).pack())
                .lock(lockscript.clone())
                .build();
            tx_helper.add_output(cell_output, Default::default());
        }
        let unsigned_tx = tx_helper
            .supply_capacity(
                &mut generator.rpc_client,
                &mut generator.indexer_client,
                lockscript,
                &generator.genesis_info,
                tx_fee,
            )
            .map_err(|err| anyhow!(err))?;
        let tx_hash = generator
            .sign_and_send_transaction(unsigned_tx, self.private_key)
            .await?;
        log::info!(
            "generate capacity cell for next round succeed: {:?}",
            tx_hash
        );
        Ok(())
    }

    async fn client_confirmed_number(&self) -> Result<u64> {
        let mut generator = self.get_generator().await?;
        let force_contracts = self
            .force_config
            .deployed_contracts
            .clone()
            .expect("force contracts deployed");
        let tip_number = get_eth_client_tip_number(
            &mut generator,
            force_contracts.light_client_cell_script.cell_script,
        )?;
        if tip_number > (CONFIRM as u64) {
            Ok(tip_number - (CONFIRM as u64))
        } else {
            Ok(0)
        }
    }

    async fn get_generator(&self) -> Result<Generator> {
        let force_contracts = self
            .force_config
            .deployed_contracts
            .clone()
            .expect("force contracts deployed");
        let mut generator_opt = None;
        for i in 0..4 {
            let generator_res = Generator::new(
                self.ckb_rpc_url.clone(),
                self.ckb_indexer_url.clone(),
                force_contracts.clone(),
            );
            match generator_res {
                Ok(g) => {
                    generator_opt = Some(g);
                    break;
                }
                Err(e) => {
                    if i < 3 {
                        log::info!("new geneartor fail {}, err: {}", i, e);
                        continue;
                    }
                    bail!("new geneartor fail after retry, err: {}", e);
                }
            }
        }
        let mut generator = generator_opt.unwrap();
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
            .await
            .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
        Ok(generator)
    }
}
