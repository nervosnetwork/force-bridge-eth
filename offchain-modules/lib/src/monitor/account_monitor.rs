use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{get_secret_key, parse_privkey};
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_eth_address, secret_key_address, Web3Client};
use anyhow::{anyhow, Result};
use ckb_types::packed::Script;
use force_sdk::cell_collector::get_all_live_cells_by_lockscript;
use secp256k1::SecretKey;
use web3::types::Address;

pub struct AccountMonitor {
    web3_client: Web3Client,
    generator: Generator,
    eth_addresses: Vec<Address>,
    ckb_lockscripts: Vec<Script>,
    alarm_url: String,
    ckb_alarm_balance: u64,
    eth_alarm_balance: u64,
    eth_balance_conservator: String,
    ckb_balance_conservator: String,
}

impl AccountMonitor {
    pub async fn new(
        config_path: String,
        network: Option<String>,
        alarm_url: String,
        ckb_alarm_balance: u64,
        eth_alarm_balance: u64,
        eth_balance_conservator: String,
        ckb_balance_conservator: String,
    ) -> Result<Self> {
        let force_config = ForceConfig::new(&config_path)?;
        let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
        let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
        let ckb_privkeys = force_config.get_ckb_private_keys(&network)?;
        let eth_privkeys: Vec<String> = force_config.get_ethereum_private_keys(&network)?;

        let mut eth_addresses: Vec<Address> = vec![];
        for eth_privkey in eth_privkeys.into_iter() {
            let eth_private_key = convert_eth_address(&eth_privkey)?;
            let eth_key = SecretKey::from_slice(&eth_private_key.0)?;
            let from = secret_key_address(&eth_key);
            eth_addresses.push(from);
        }
        let mut ckb_lockscripts: Vec<Script> = vec![];
        for ckb_privkey in ckb_privkeys.into_iter() {
            let privkey = get_secret_key(&ckb_privkey)?;
            let lockscript = parse_privkey(&privkey);
            ckb_lockscripts.push(lockscript);
        }

        let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
        Ok(AccountMonitor {
            web3_client: Web3Client::new(eth_rpc_url),
            generator: Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default())
                .map_err(|e| anyhow!("failed to crate generator: {}", e))?,
            eth_addresses,
            ckb_lockscripts,
            alarm_url,
            ckb_alarm_balance,
            eth_alarm_balance,
            eth_balance_conservator,
            ckb_balance_conservator,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut msg = " ".to_string();
        for eth_address in self.eth_addresses.iter() {
            let balance = self
                .web3_client
                .client()
                .eth()
                .balance(*eth_address, None)
                .await?;
            let mut eth_balance_msg =
                format!("{} balance is : {:?}", hex::encode(eth_address), balance);
            if balance.as_u64() < self.eth_alarm_balance {
                eth_balance_msg = format!("{} @{}", eth_balance_msg, self.eth_balance_conservator)
            }
            log::info!("eth balance info : {}", eth_balance_msg);
            msg = format!("{} {} %0A", msg, eth_balance_msg);
        }

        for ckb_lockscript in self.ckb_lockscripts.iter() {
            let live_cells = get_all_live_cells_by_lockscript(
                &mut self.generator.indexer_client,
                ckb_lockscript.clone(),
            )
            .map_err(|err| anyhow!(err))?;
            let mut capacity = 0;
            capacity += live_cells
                .iter()
                .map(|c| c.output.capacity.value())
                .sum::<u64>();

            let mut ckb_balance_msg = format!(
                "{} balance is : {:x}",
                ckb_lockscript.as_reader().calc_script_hash(),
                capacity,
            );
            if capacity < self.ckb_alarm_balance {
                ckb_balance_msg = format!("{} @{}", ckb_balance_msg, self.ckb_balance_conservator)
            }
            log::info!("ckb balance info : {}", ckb_balance_msg);
            msg = format!("{} {} %0A", msg, ckb_balance_msg);
        }

        let res = reqwest::get(format!("{}{}", self.alarm_url, msg).as_str())
            .await?
            .text()
            .await?;
        log::info!("{:?}", res);
        Ok(())
    }
}
