use config::{Config, ConfigError, Environment, File};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct OutpointConf {
    pub tx_hash: String,
    pub index: u32,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ScriptConf {
    pub code_hash: String,
    pub outpoint: OutpointConf,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ScriptsConf {
    pub lockscript: ScriptConf,
    pub typescript: ScriptConf,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CellScript {
    pub cell_script: String,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Settings {
    pub eth_token_locker_addr: String,
    pub eth_ckb_chain_addr: String,
    pub bridge_lockscript: ScriptConf,
    pub bridge_typescript: ScriptConf,
    pub light_client_typescript: ScriptConf,
    pub light_client_lockscript: ScriptConf,
    pub recipient_typescript: ScriptConf,
    pub sudt: ScriptConf,
    // pub replay_resist_lockscript: ScriptConf,
    pub dag_merkle_roots: OutpointConf,
    pub light_client_cell_script: CellScript,
}

impl Settings {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(config_path))?;
        s.merge(Environment::with_prefix("app"))?;
        s.try_into()
    }

    pub fn write(&self, config_path: &str) -> Result<(), String> {
        let s = toml::to_string(self).map_err(|e| format!("toml serde error: {}", e))?;
        std::fs::write(config_path, &s)
            .map_err(|e| format!("fail to write scripts config. err: {}", e))?;
        Ok(())
    }
}
