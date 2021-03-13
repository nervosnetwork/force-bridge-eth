use crate::dapp::db::indexer::get_height_info;
use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{get_secret_key, parse_cell, parse_merkle_cell_data, parse_privkey};
use crate::util::eth_util::{convert_eth_address, secret_key_address, Web3Client};
use anyhow::{anyhow, bail, Result};
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::packed::Script;
use ethereum_types::{H160, U256};
use force_sdk::cell_collector::{get_all_live_cells_by_lockscript, get_live_cell_by_typescript};
use secp256k1::SecretKey;
use sqlx::MySqlPool;
use std::ops::{Div, Sub};

pub struct RelayMonitor {
    web3_client: Web3Client,
    generator: Generator,
    alarm_url: String,
    mode: String,
    ckb_alarm_number: u64,
    eth_alarm_number: u64,
    header_args: Option<HeaderMonitorArgs>,
    indexer_args: Option<IndexerMonitorArgs>,
    account_monitor_args: AccountMonitorArgs,
}

#[derive(Debug, Clone)]
pub struct AccountMonitorArgs {
    eth_addresses: Vec<H160>,
    ckb_lockscripts: Vec<Script>,
    ckb_alarm_balance: u64,
    eth_alarm_balance: u64,
    eth_balance_conservator: String,
    ckb_balance_conservator: String,
    ckb_network: NetworkType,
}

impl AccountMonitorArgs {
    pub async fn new(
        ckb_privkeys: Vec<String>,
        eth_privkeys: Vec<String>,
        ckb_alarm_balance: u64,
        eth_alarm_balance: u64,
        eth_balance_conservator: String,
        ckb_balance_conservator: String,
        config_network: &Option<String>,
    ) -> Result<Self> {
        let ckb_network = if let Some(network) = config_network {
            match network.as_str() {
                "mainnet" => NetworkType::Mainnet,
                "ropsten" => NetworkType::Testnet,
                "rinkeby" => NetworkType::Testnet,
                "docker-dev-chain" => NetworkType::Dev,
                _ => NetworkType::Dev,
            }
        } else {
            NetworkType::Dev
        };

        let mut eth_addresses: Vec<H160> = vec![];
        for eth_privkey in eth_privkeys.into_iter() {
            let eth_private_key =
                ethereum_types::H256::from_slice(hex::decode(eth_privkey)?.as_slice());
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

        Ok(AccountMonitorArgs {
            eth_addresses,
            ckb_lockscripts,
            ckb_alarm_balance,
            eth_alarm_balance,
            eth_balance_conservator,
            ckb_balance_conservator,
            ckb_network,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HeaderMonitorArgs {
    script: Script,
    contract_addr: H160,
    eth_header_conservator: Vec<String>,
    ckb_header_conservator: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexerMonitorArgs {
    eth_indexer_conservator: Vec<String>,
    ckb_indexer_conservator: Vec<String>,
    db: MySqlPool,
}

impl RelayMonitor {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ckb_rpc_url: String,
        ckb_indexer_url: String,
        eth_rpc_url: String,
        ckb_alarm_number: u64,
        eth_alarm_number: u64,
        alarm_url: String,
        mode: String,
        cell: String,
        eth_ckb_chain_addr: String,
        eth_header_conservator: Option<Vec<String>>,
        ckb_header_conservator: Option<Vec<String>>,
        eth_indexer_conservator: Option<Vec<String>>,
        ckb_indexer_conservator: Option<Vec<String>>,
        db_path: Option<String>,
        account_monitor_args: AccountMonitorArgs,
    ) -> Result<RelayMonitor> {
        let web3_client = Web3Client::new(eth_rpc_url);
        let generator = Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default())
            .map_err(|e| anyhow!("failed to crate generator: {}", e))?;

        let mut header_args: Option<HeaderMonitorArgs> = None;
        let mut indexer_args: Option<IndexerMonitorArgs> = None;
        log::info!("mode : {}", mode);
        match mode.as_str() {
            "all" => {
                header_args = Option::from(HeaderMonitorArgs {
                    script: parse_cell(&cell)
                        .map_err(|e| anyhow!("get typescript fail {:?}", e))?,
                    contract_addr: convert_eth_address(&eth_ckb_chain_addr)?,
                    eth_header_conservator: eth_header_conservator.ok_or_else(|| {
                        anyhow!("the eth_header_conservator can not be none in all mode")
                    })?,
                    ckb_header_conservator: ckb_header_conservator.ok_or_else(|| {
                        anyhow!("the ckb_header_conservator can not be none in all mode")
                    })?,
                });
                indexer_args = Option::from(IndexerMonitorArgs {
                    eth_indexer_conservator: eth_indexer_conservator.ok_or_else(|| {
                        anyhow!("the eth_indexer_conservator can not be none in all mode")
                    })?,
                    ckb_indexer_conservator: ckb_indexer_conservator.ok_or_else(|| {
                        anyhow!("the ckb_indexer_conservator can not be none in all mode")
                    })?,
                    db: MySqlPool::connect(
                        db_path
                            .ok_or_else(|| anyhow!("the db_path can not be none in all mode "))?
                            .as_str(),
                    )
                    .await?,
                });
            }
            "header" => {
                header_args = Option::from(HeaderMonitorArgs {
                    script: parse_cell(&cell)
                        .map_err(|e| anyhow!("get typescript fail {:?}", e))?,
                    contract_addr: convert_eth_address(&eth_ckb_chain_addr)?,
                    eth_header_conservator: eth_header_conservator.ok_or_else(|| {
                        anyhow!("the eth_header_conservator can not be none in header mode")
                    })?,
                    ckb_header_conservator: ckb_header_conservator.ok_or_else(|| {
                        anyhow!("the ckb_header_conservator can not be none in header mode")
                    })?,
                });
            }
            "indexer" => {
                indexer_args = Option::from(IndexerMonitorArgs {
                    eth_indexer_conservator: eth_indexer_conservator.ok_or_else(|| {
                        anyhow!("the eth_indexer_conservator can not be none in indexer mode")
                    })?,
                    ckb_indexer_conservator: ckb_indexer_conservator.ok_or_else(|| {
                        anyhow!("the ckb_indexer_conservator can not be none in indexer mode")
                    })?,
                    db: MySqlPool::connect(
                        db_path
                            .ok_or_else(|| anyhow!("the db_path can not be none in indexer mode"))?
                            .as_str(),
                    )
                    .await?,
                });
            }
            _ => bail!("the mode arg is wrong in constructor "),
        }

        return Ok(RelayMonitor {
            web3_client,
            generator,
            alarm_url,
            mode,
            ckb_alarm_number,
            eth_alarm_number,
            header_args,
            indexer_args,
            account_monitor_args,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn start(&mut self) -> Result<()> {
        let mut msg = " ".to_string();
        let header_monitor_msg = self.get_header_monitor_info().await?;
        let account_monitor_msg = self.get_account_monitor_info().await?;
        msg = format!(
            "{} {} %0A {} %0A",
            msg, header_monitor_msg, account_monitor_msg
        );
        let res = reqwest::get(format!("{}{}", self.alarm_url, msg).as_str())
            .await?
            .text()
            .await?;
        log::info!("{:?}", res);
        Ok(())
    }

    pub async fn get_header_monitor_info(&mut self) -> Result<String> {
        let mut msg = " ".to_string();
        match self.mode.as_str() {
            "all" => {
                match self.indexer_args.clone() {
                    None => {
                        bail!("the indexer args can not be none in all mode");
                    }
                    Some(args) => {
                        let index_msg = self.get_indexer_height(args).await?;
                        msg = format!("{} {} ", msg, index_msg);
                    }
                };

                match self.header_args.clone() {
                    None => {
                        bail!("the header args can not be none in all mode");
                    }
                    Some(args) => {
                        let height_msg = self.get_header_height(args).await?;
                        msg = format!("{} %0A{} ", msg, height_msg);
                    }
                };
            }
            "header" => match self.header_args.clone() {
                None => {
                    bail!("the header args can not be none in header mode");
                }
                Some(args) => {
                    let height_msg = self.get_header_height(args).await?;
                    msg = format!("{} {} ", msg, height_msg);
                }
            },
            "indexer" => match self.indexer_args.clone() {
                None => {
                    bail!("the indexer args can not be none in indexer mode");
                }
                Some(args) => {
                    let index_msg = self.get_indexer_height(args).await?;
                    msg = format!("{} {} ", msg, index_msg);
                }
            },

            _ => bail!("the mode arg is wrong in monitor"),
        }
        Ok(msg)
    }
    pub async fn get_account_monitor_info(&mut self) -> Result<String> {
        let mut msg = " ".to_string();
        let eth_decimal: U256 = U256::from(10u128.pow(18));
        for (index, eth_address) in self.account_monitor_args.eth_addresses.iter().enumerate() {
            let balance = self
                .web3_client
                .client()
                .eth()
                .balance(*eth_address, None)
                .await?
                .div(eth_decimal);
            let mut eth_balance_msg = format!(
                "ethereum_private_keys[{:?}] {} balance is : {:?} eth",
                index,
                hex::encode(eth_address),
                balance
            );
            if balance.as_u64() < self.account_monitor_args.eth_alarm_balance {
                eth_balance_msg = format!(
                    "{} @{}",
                    eth_balance_msg, self.account_monitor_args.eth_balance_conservator
                )
            }
            log::info!("eth balance info : {}", eth_balance_msg);
            msg = format!("{} {} %0A", msg, eth_balance_msg);
        }

        for (index, ckb_lockscript) in self.account_monitor_args.ckb_lockscripts.iter().enumerate()
        {
            let live_cells = get_all_live_cells_by_lockscript(
                &mut self.generator.indexer_client,
                ckb_lockscript.clone(),
            )
            .map_err(|err| anyhow!(err))?;
            let mut capacity = 0;
            capacity += live_cells
                .iter()
                .map(|c| c.output.capacity.value())
                .sum::<u64>()
                .div(10u64.pow(8));

            let from_addr_payload: AddressPayload = ckb_lockscript.clone().into();
            let from_addr = Address::new(self.account_monitor_args.ckb_network, from_addr_payload);
            let mut ckb_balance_msg = format!(
                "ckb_private_keys[{:?}] {:?} balance is : {:?} ckb",
                index,
                from_addr.to_string(),
                capacity,
            );
            if capacity < self.account_monitor_args.ckb_alarm_balance {
                ckb_balance_msg = format!(
                    "{} @{}",
                    ckb_balance_msg, self.account_monitor_args.ckb_balance_conservator
                )
            }
            log::info!("ckb balance info : {}", ckb_balance_msg);
            msg = format!("{} {} %0A", msg, ckb_balance_msg);
        }
        Ok(msg)
    }
    pub async fn get_header_height(&mut self, args: HeaderMonitorArgs) -> Result<String> {
        let ckb_light_client_height = self
            .web3_client
            .get_contract_height("latestBlockNumber", args.contract_addr)
            .await?;
        let ckb_current_height = self
            .generator
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;

        let eth_current_height = self.web3_client.client().eth().block_number().await?;

        let cell = get_live_cell_by_typescript(&mut self.generator.indexer_client, args.script)
            .map_err(|e| anyhow!("get live cell fail: {}", e))?
            .ok_or_else(|| anyhow!("eth header cell not exist"))?;

        let (_, eth_light_client_height, _) =
            parse_merkle_cell_data(cell.output_data.as_bytes().to_vec())?;

        let ckb_diff = ckb_current_height - ckb_light_client_height;
        let eth_diff = eth_current_height.sub(eth_light_client_height).as_u64();

        let mut msg = format!("ckb light client height : {:?}  %0A ckb current height : {:?}  %0A eth light client height : {:?}  %0A eth current height : {:?} %0A ckb height diff is {:?}, eth height diff is {:?} %0A ", ckb_light_client_height, ckb_current_height, eth_light_client_height, eth_current_height, ckb_diff, eth_diff);

        if self.ckb_alarm_number < ckb_diff {
            for conservator in args.eth_header_conservator.iter() {
                msg = format!("{} @{} ", msg, conservator,);
            }
            msg = format!("{} %0A ", msg);
        }

        if self.eth_alarm_number < eth_diff {
            for conservator in args.ckb_header_conservator.iter() {
                msg = format!("{} @{} ", msg, conservator,);
            }
            msg = format!("{} %0A ", msg);
        }

        Ok(msg)
    }
    pub async fn get_indexer_height(&mut self, args: IndexerMonitorArgs) -> Result<String> {
        let eth_height_info = get_height_info(&args.db, 1 as u8).await?;
        let ckb_height_info = get_height_info(&args.db, 2 as u8).await?;

        let ckb_current_height = self
            .generator
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;
        let eth_current_height = self.web3_client.client().eth().block_number().await?;

        let ckb_diff = ckb_current_height - ckb_height_info.height;
        let eth_diff = eth_current_height.as_u64() - eth_height_info.height;

        let mut msg = format!("ckb chain height: {:?}  %0A ckb light client height in db record: {:?}  %0A ckb indexer height in db record : {:?}  %0A  eth chain height : {:?}  %0A  eth light client height in db record : {:?}  %0A eth indexer height in db record : {:?} %0A ckb height diff in db is {:?}, eth height diff in db is {:?} %0A ", ckb_current_height, ckb_height_info.client_height,ckb_height_info.height, eth_current_height, eth_height_info.client_height, eth_height_info.height, ckb_diff, eth_diff);

        if self.ckb_alarm_number < ckb_diff {
            for conservator in args.ckb_indexer_conservator.iter() {
                msg = format!("{} @{} ", msg, conservator,);
            }
            msg = format!("{} %0A ", msg);
        }

        if self.eth_alarm_number < eth_diff {
            for conservator in args.eth_indexer_conservator.iter() {
                msg = format!("{} @{} ", msg, conservator,);
            }
            msg = format!("{} %0A ", msg);
        }

        Ok(msg)
    }
}
