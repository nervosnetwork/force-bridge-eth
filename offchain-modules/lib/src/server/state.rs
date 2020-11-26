use crate::util::ckb_util::Generator;
use crate::util::config::{DeployedContracts, ForceConfig};
use anyhow::{anyhow, Result};
use force_sdk::util::ensure_indexer_sync;
use shellexpand::tilde;

#[derive(Clone)]
pub struct DappState {
    pub config_path: String,
    pub network: Option<String>,
    pub private_key_path: String,
    pub deployed_contracts: DeployedContracts,
    pub indexer_url: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
}

impl DappState {
    pub fn new(
        config_path: String,
        network: Option<String>,
        private_key_path: String,
    ) -> Result<Self> {
        let config_path = tilde(config_path.as_str()).into_owned();
        let force_config = ForceConfig::new(config_path.as_str())?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let indexer_url = force_config.get_ckb_indexer_url(&network)?;
        Ok(Self {
            private_key_path,
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            deployed_contracts: force_config
                .deployed_contracts
                .expect("contracts should be deployed"),
            network,
        })
    }

    pub fn get_generator(&self) -> Result<Generator> {
        let mut generator = Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.deployed_contracts.clone(),
        )
        .map_err(|e| anyhow!("new geneartor fail, err: {}", e))?;
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
            .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
        Ok(generator)
    }
}
