use anyhow::{anyhow, Result};
use config::{Config, ConfigError, Environment, File};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;
use std::env;

pub const DEFAULT_CONFIG_PATH: &str = "conf/config.toml";

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct SignServerConfig {
    pub config_path: String,
    pub ckb_key_path: String,
    pub eth_key_path: String,
    pub cell_script: String,
    pub eth_rpc_url: String,
    pub ckb_rpc_url: String,
    pub ckb_indexer_url: String,
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

pub fn get_config_path() -> String {
    let args_: Vec<String> = env::args().collect();
    for i in 0..args_.len() {
        match args_[i].as_str() {
            "--config-path" => {
                if args_.clone().len() <= i + 1 || args_[i + 1].clone().starts_with("--") {
                    return DEFAULT_CONFIG_PATH.to_string();
                }
                return args_[i + 1].clone();
            }
            _ => {}
        }
    }
    DEFAULT_CONFIG_PATH.to_string()
}
