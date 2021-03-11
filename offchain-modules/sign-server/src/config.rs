use anyhow::{anyhow, Result};
use config::{Config, ConfigError, Environment, File};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct SignServerConfig {
    pub db_path: String,
    pub ckb_private_key_path: String,
    pub eth_private_key_path: String,
}

impl SignServerConfig {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let config_path = tilde(config_path).into_owned();
        let mut s = Config::new();
        s.merge(File::with_name(config_path.as_str()))?;
        s.merge(Environment::with_prefix("app"))?;
        s.try_into()
    }

    pub fn write(&self, config_path: &str) -> Result<()> {
        let s = toml::to_string_pretty(self).map_err(|e| anyhow!("toml serde error: {}", e))?;
        println!("{:?}", s);

        let parent_path = std::path::Path::new(config_path)
            .parent()
            .ok_or_else(|| anyhow!("invalid config file path: {}", config_path))?;
        println!("{:?}", parent_path);
        std::fs::create_dir_all(parent_path)
            .map_err(|e| anyhow!("fail to create config path. err: {}", e))?;
        std::fs::write(config_path, &s)
            .map_err(|e| anyhow!("fail to write scripts config. err: {}", e))
    }
}
