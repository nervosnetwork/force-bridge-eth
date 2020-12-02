use crate::util::ckb_util::{parse_privkey_path, Generator};
use crate::util::config::{DeployedContracts, ForceConfig};
use anyhow::{anyhow, Result};
use force_sdk::util::ensure_indexer_sync;
use secp256k1::SecretKey;
use shellexpand::tilde;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DappState {
    pub config_path: String,
    pub network: Option<String>,
    pub ckb_private_key_path: String,
    pub eth_private_key_path: String,
    pub from_privkey: SecretKey,
    pub deployed_contracts: DeployedContracts,
    pub indexer_url: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub db: SqlitePool,
}

impl DappState {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        ckb_private_key_path: String,
        eth_private_key_path: String,
        db_path: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let indexer_url = force_config.get_ckb_indexer_url(&network)?;
        let from_privkey =
            parse_privkey_path(ckb_private_key_path.as_str(), &force_config, &network)?;
        Ok(Self {
            ckb_private_key_path,
            eth_private_key_path,
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            from_privkey,
            deployed_contracts: force_config
                .deployed_contracts
                .expect("contracts should be deployed"),
            network,
            db: SqlitePool::connect(&db_path).await?,
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
}
