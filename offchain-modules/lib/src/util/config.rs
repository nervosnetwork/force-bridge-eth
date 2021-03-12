use anyhow::{anyhow, Result};
use config::{Config, ConfigError, Environment, File};
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;
use std::path::PathBuf;
use toml::value::{Table, Value};

const CKB_PRIVATE_KEYS: [&str; 6] = [
    "a800c82df5461756ae99b5c6677d019c98cc98c7786b80d7b2e77256e46ea1fe",
    "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc",
    "63d86723e08f0f813a36ce6aa123bb2289d90680ae1e99d4de8cdb334553f24d",
    "a6b8e0cbadda5c0d91cf82d1e8d8120b755aa06bc49030ca6e8392458c65fc80",
    "13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "a6b023fec4fc492c23c0e999ab03b01a6ca5524a3560725887a8de4362f9c9cc",
];
const ETHEREUM_PRIVATE_KEYS: [&str; 6] = [
    "c4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a",
    "719e94ec5d2ecef67b5878503ffd6e1e0e2fe7a52ddd55c436878cb4d52d376d",
    "627ed509aa9ef55858d01453c62f44287f639a4fa5a444af150f333b6010a3b6",
    "49e7074797d83cbb93b23877f99a8cecd6f79181f1236f095671017b2edc64c2",
    "6e51216cbb2fe170368da49e82b22f02b999204730c858482d0e84a9083005ac",
    "ca2e37b4f2e4a122cc86c401e3d1da3841c525f01b5b249dcdcd69e2f086d576",
];

#[allow(clippy::too_many_arguments)]
pub async fn init_config(
    is_force: bool,
    project_path: String,
    config_path: String,
    default_network: String,
    ckb_rpc_url: String,
    ckb_indexer_url: String,
    ethereum_rpc_url: String,
    eth_rocksdb_path: String,
    ckb_rocksdb_path: String,
) -> Result<()> {
    let config_path = tilde(config_path.as_str()).into_owned();
    if std::path::Path::new(&config_path).exists() && !is_force {
        println!(
            "force-cli-config already exists at {}, so this command do nothing. if you want to overwrite it, use `-f`",
            &config_path
        );
        return Ok(());
    }
    let mut network_config = Table::new();
    network_config.insert("ckb_rpc_url".to_string(), Value::String(ckb_rpc_url));
    network_config.insert(
        "ckb_indexer_url".to_string(),
        Value::String(ckb_indexer_url),
    );
    network_config.insert(
        "ethereum_rpc_url".to_string(),
        Value::String(ethereum_rpc_url),
    );
    let (ckb_private_keys, ethereum_private_keys) = if default_network == "docker-dev-chain" {
        let ckb_private_keys = CKB_PRIVATE_KEYS
            .to_vec()
            .into_iter()
            .map(|v| Value::String(v.to_string()))
            .collect::<Vec<Value>>();
        let ethereum_private_keys = ETHEREUM_PRIVATE_KEYS
            .to_vec()
            .into_iter()
            .map(|v| Value::String(v.to_string()))
            .collect::<Vec<Value>>();
        (ckb_private_keys, ethereum_private_keys)
    } else {
        (Vec::<Value>::new(), Vec::<Value>::new())
    };
    network_config.insert(
        "ckb_private_keys".to_string(),
        Value::Array(ckb_private_keys),
    );
    network_config.insert(
        "ethereum_private_keys".to_string(),
        Value::Array(ethereum_private_keys),
    );

    let mut networks_config = Table::new();
    networks_config.insert(default_network.clone(), Value::Table(network_config));
    let force_cli_config = ForceConfig {
        project_path,
        eth_rocksdb_path,
        ckb_rocksdb_path,
        default_network,
        networks_config,
        deployed_contracts: None,
    };
    force_cli_config.write(config_path.as_str())
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ForceConfig {
    pub project_path: String,
    pub eth_rocksdb_path: String,
    pub ckb_rocksdb_path: String,
    pub default_network: String,
    pub deployed_contracts: Option<DeployedContracts>,
    #[serde(serialize_with = "toml::ser::tables_last")]
    pub networks_config: Table,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct NetworkConfig {
    pub ckb_rpc_url: String,
    pub ckb_indexer_url: String,
    pub ethereum_rpc_url: String,
    pub ckb_private_keys: Vec<Value>,
    pub ethereum_private_keys: Vec<Value>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct DeployedContracts {
    pub eth_token_locker_addr: String,
    pub eth_ckb_chain_addr: String,
    pub bridge_lockscript: ScriptConf,
    pub bridge_typescript: ScriptConf,
    pub light_client_typescript: ScriptConf,
    // pub light_client_lockscript: ScriptConf,
    pub recipient_typescript: ScriptConf,
    pub simple_bridge_typescript: ScriptConf,
    pub sudt: ScriptConf,
    pub light_client_cell_script: CellScript,
    pub multisig_address: MultisigConf,
    pub ckb_relay_mutlisig_threshold: CKBRelayMultisigConf,
    pub pw_locks: PwLocks,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct MultisigConf {
    pub addresses: Vec<String>,
    pub require_first_n: u8,
    pub threshold: u8,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CKBRelayMultisigConf {
    pub threshold: usize,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct PwLocks {
    pub inner: Vec<OutpointConf>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct OutpointConf {
    pub tx_hash: String,
    pub index: u32,
    pub dep_type: u8,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ScriptConf {
    pub code_hash: String,
    pub hash_type: u8,
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

impl ForceConfig {
    pub fn new(config_path: &str) -> Result<Self, ConfigError> {
        let config_path = tilde(config_path).into_owned();
        let mut s = Config::new();
        s.merge(File::with_name(config_path.as_str()))?;
        s.merge(Environment::with_prefix("app"))?;
        s.try_into()
    }

    pub fn get_network_config(&self, network: &Option<String>) -> Result<NetworkConfig> {
        let network = if let Some(network) = network {
            network
        } else {
            &self.default_network
        };
        let network_config = self.networks_config.get(network).ok_or_else(|| {
            anyhow!(
                "invalid config file: networks_config.{} not existed",
                self.default_network
            )
        })?;
        if let Value::Table(network_config) = network_config {
            let ckb_rpc_url = network_config
                .get("ckb_rpc_url")
                .ok_or_else(|| anyhow!("invalid config file: ckb rpc url not existed"))?;
            let ckb_rpc_url = if let Value::String(ckb_rpc_url) = ckb_rpc_url {
                ckb_rpc_url.to_owned()
            } else {
                return Err(anyhow!("ckb rpc url should be Value::String"));
            };
            let ckb_indexer_url = network_config
                .get("ckb_indexer_url")
                .ok_or_else(|| anyhow!("invalid config file: ckb indexer url not existed"))?;
            let ckb_indexer_url = if let Value::String(ckb_indexer_url) = ckb_indexer_url {
                ckb_indexer_url.to_owned()
            } else {
                return Err(anyhow!("ckb indexer url should be Value::String"));
            };
            let ethereum_rpc_url = network_config
                .get("ethereum_rpc_url")
                .ok_or_else(|| anyhow!("invalid config file: ethereum rpc url not existed"))?;
            let ethereum_rpc_url = if let Value::String(ethereum_rpc_url) = ethereum_rpc_url {
                ethereum_rpc_url.to_owned()
            } else {
                return Err(anyhow!("ethereum rpc url should be Value::String"));
            };
            let ckb_private_keys = network_config
                .get("ckb_private_keys")
                .ok_or_else(|| anyhow!("invalid config file: ckb_private_keys not existed"))?;
            let ckb_private_keys = if let Value::Array(ckb_private_keys) = ckb_private_keys {
                ckb_private_keys.to_owned()
            } else {
                return Err(anyhow!("ckb_private_keys should be Value::Array"));
            };
            let ethereum_private_keys = network_config
                .get("ethereum_private_keys")
                .ok_or_else(|| anyhow!("invalid config file: ethereum_private_keys not existed"))?;
            let ethereum_private_keys =
                if let Value::Array(ethereum_private_keys) = ethereum_private_keys {
                    ethereum_private_keys.to_owned()
                } else {
                    return Err(anyhow!("ethereum_private_keys should be Value::Array"));
                };
            Ok(NetworkConfig {
                ckb_rpc_url,
                ckb_indexer_url,
                ethereum_rpc_url,
                ckb_private_keys,
                ethereum_private_keys,
            })
        } else {
            Err(anyhow!("chain config should be Value::Table"))
        }
    }

    pub fn get_ckb_rpc_url(&self, network: &Option<String>) -> Result<String> {
        let chain_config = self.get_network_config(network)?;
        Ok(chain_config.ckb_rpc_url)
    }

    pub fn get_ckb_indexer_url(&self, network: &Option<String>) -> Result<String> {
        let chain_config = self.get_network_config(network)?;
        Ok(chain_config.ckb_indexer_url)
    }

    pub fn get_ethereum_rpc_url(&self, network: &Option<String>) -> Result<String> {
        let chain_config = self.get_network_config(network)?;
        Ok(chain_config.ethereum_rpc_url)
    }

    pub fn get_ckb_private_keys(&self, network: &Option<String>) -> Result<Vec<String>> {
        self.get_network_config(network)?
            .ckb_private_keys
            .into_iter()
            .map(|v| {
                if let Value::String(k) = v {
                    Ok(k)
                } else {
                    Err(anyhow!("ckb private key should be string"))
                }
            })
            .collect()
    }

    pub fn get_ethereum_private_keys(&self, network: &Option<String>) -> Result<Vec<String>> {
        self.get_network_config(network)?
            .ethereum_private_keys
            .into_iter()
            .map(|v| {
                if let Value::String(k) = v {
                    Ok(k)
                } else {
                    Err(anyhow!("ethereum private key should be string"))
                }
            })
            .collect()
    }

    pub fn get_ckb_script_bin_path(&self) -> Result<PathBuf> {
        let project_path = std::path::Path::new(self.project_path.as_str());
        let ckb_script_bin_path =
            project_path.join(std::path::Path::new("ckb-contracts/build/release"));
        Ok(ckb_script_bin_path)
    }

    pub fn get_bridge_typescript_bin_path(&self) -> Result<String> {
        let bridge_typescript_bin_path = self
            .get_ckb_script_bin_path()?
            .join(std::path::Path::new("eth-bridge-typescript"));
        bridge_typescript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn get_simple_bridge_typescript_bin_path(&self) -> Result<String> {
        let simple_bridge_typescript_bin_path = self
            .get_ckb_script_bin_path()?
            .join(std::path::Path::new("simple-eth-bridge-typescript"));
        simple_bridge_typescript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn get_bridge_lockscript_bin_path(&self) -> Result<String> {
        let bridge_lockscript_bin_path = self
            .get_ckb_script_bin_path()?
            .join(std::path::Path::new("eth-bridge-lockscript"));
        bridge_lockscript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn get_light_client_typescript_bin_path(&self) -> Result<String> {
        let light_client_typescript_bin_path = self
            .get_ckb_script_bin_path()?
            .join(std::path::Path::new("eth-light-client-typescript"));
        light_client_typescript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn get_light_client_lockscript_bin_path(&self) -> Result<String> {
        let light_client_lockscript_bin_path = self
            .get_ckb_script_bin_path()?
            .join(std::path::Path::new("eth-light-client-lockscript"));
        light_client_lockscript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn get_recipient_typescript_bin_path(&self) -> Result<String> {
        let recipient_typescript_bin_path = self
            .get_ckb_script_bin_path()?
            .join(std::path::Path::new("eth-recipient-typescript"));
        recipient_typescript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn get_sudt_typescript_bin_path(&self) -> Result<String> {
        let project_path = std::path::Path::new(self.project_path.as_str());
        let sudt_typescript_bin_path =
            project_path.join(std::path::Path::new("ckb-contracts/tests/deps/simple_udt"));
        sudt_typescript_bin_path
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!(format!("{:?}", e)))
    }

    pub fn write(&self, config_path: &str) -> Result<()> {
        let s = toml::to_string_pretty(self).map_err(|e| anyhow!("toml serde error: {}", e))?;
        let parent_path = std::path::Path::new(config_path)
            .parent()
            .ok_or_else(|| anyhow!("invalid config file path: {}", config_path))?;
        std::fs::create_dir_all(parent_path)
            .map_err(|e| anyhow!("fail to create config path. err: {}", e))?;
        std::fs::write(config_path, &s)
            .map_err(|e| anyhow!("fail to write scripts config. err: {}", e))
    }
}
