use crate::util::ckb_util::Generator;
use crate::util::settings::Settings;
use anyhow::{anyhow, Result};
use force_sdk::util::ensure_indexer_sync;

#[derive(Clone)]
pub struct DappState {
    pub config_path: String,
    pub indexer_url: String,
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub settings: Settings,
    pub private_key_path: String,
}

impl DappState {
    pub fn new(
        config_path: String,
        indexer_url: String,
        ckb_rpc_url: String,
        eth_rpc_url: String,
        private_key_path: String,
    ) -> Result<Self> {
        let dapp_settings = Settings::new(config_path.as_str())?;
        Ok(Self {
            private_key_path,
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            settings: dapp_settings,
        })
    }

    pub fn get_generator(&self) -> Result<Generator> {
        let mut generator = Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.settings.clone(),
        )
        .map_err(|e| anyhow!("new geneartor fail, err: {}", e))?;
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
            .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
        Ok(generator)
    }
}
