use crate::dapp::db::indexer::get_height_info;
use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{parse_cell, parse_main_chain_headers};
use crate::util::eth_util::{convert_eth_address, Web3Client};
use anyhow::{anyhow, bail, Result};
use ckb_types::packed::Script;
use ethereum_types::H160;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use sqlx::MySqlPool;
use std::ops::Sub;

pub struct RelayMonitor {
    web3_client: Web3Client,
    generator: Generator,
    alarm_url: String,
    mode: String,
    ckb_alarm_number: u64,
    eth_alarm_number: u64,
    header_args: Option<HeaderMonitorArgs>,
    indexer_args: Option<IndexerMonitorArgs>,
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
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn start(&mut self) -> Result<()> {
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

        let res = reqwest::get(format!("{}{}", self.alarm_url, msg).as_str())
            .await?
            .text()
            .await?;
        log::info!("{:?}", res);
        Ok(())
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

        let (un_confirmed_headers, _) =
            parse_main_chain_headers(cell.output_data.as_bytes().to_vec())
                .map_err(|e| anyhow!("parse header data fail: {}", e))?;

        let best_header = un_confirmed_headers
            .last()
            .ok_or_else(|| anyhow!("header is none"))?;
        let eth_light_client_height = best_header
            .number
            .ok_or_else(|| anyhow!("header number is none"))?;

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
        let height_info = get_height_info(&args.db).await?;
        let ckb_db_diff = if height_info.ckb_height > height_info.ckb_client_height {
            height_info.ckb_height - height_info.ckb_client_height
        } else {
            height_info.ckb_client_height - height_info.ckb_height
        };
        let eth_db_diff = if height_info.eth_height > height_info.eth_client_height {
            height_info.eth_height - height_info.eth_client_height
        } else {
            height_info.eth_client_height - height_info.eth_height
        };

        let mut msg = format!("ckb light client height in db record: {:?}  %0A ckb indexer height in db record : {:?}  %0A eth light client height in db record : {:?}  %0A eth indexer height in db record : {:?} %0A ckb height diff in db is {:?}, eth height diff in db is {:?} %0A ", height_info.ckb_client_height, height_info.ckb_height, height_info.eth_client_height, height_info.eth_height, ckb_db_diff, eth_db_diff);

        if self.ckb_alarm_number < ckb_db_diff {
            for conservator in args.ckb_indexer_conservator.iter() {
                msg = format!("{} @{} ", msg, conservator,);
            }
            msg = format!("{} %0A ", msg);
        }

        if self.eth_alarm_number < eth_db_diff {
            for conservator in args.eth_indexer_conservator.iter() {
                msg = format!("{} @{} ", msg, conservator,);
            }
            msg = format!("{} %0A ", msg);
        }

        Ok(msg)
    }
}
