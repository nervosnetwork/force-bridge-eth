use crate::util::ckb_tx_generator::Generator;
use crate::util::ckb_util::{parse_cell, parse_main_chain_headers};
use crate::util::eth_util::{convert_eth_address, Web3Client};
use anyhow::{anyhow, Result};
use force_sdk::cell_collector::get_live_cell_by_typescript;

#[allow(clippy::too_many_arguments)]
pub async fn relay_monitor(
    ckb_rpc_url: String,
    ckb_indexer_url: String,
    eth_rpc_url: String,
    eth_ckb_chain_addr: String,
    cell: String,
    ckb_alarm_number: u64,
    eth_alarm_number: u64,
    alarm_url: String,
    ckb_conservator: Vec<String>,
    eth_conservator: Vec<String>,
) -> Result<()> {
    let contract_addr = convert_eth_address(&eth_ckb_chain_addr)?;
    let mut web3_client = Web3Client::new(eth_rpc_url);
    let mut generator = Generator::new(ckb_rpc_url, ckb_indexer_url, Default::default())
        .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
    let ckb_light_client_height = web3_client
        .get_contract_height("latestBlockNumber", contract_addr)
        .await?;
    let ckb_current_height = generator
        .rpc_client
        .get_tip_block_number()
        .map_err(|e| anyhow!("failed to get ckb current height : {}", e))?;

    let eth_current_height = web3_client.client().eth().block_number().await?;

    let typescript = parse_cell(&cell).map_err(|e| anyhow!("get typescript fail {:?}", e))?;

    let cell = get_live_cell_by_typescript(&mut generator.indexer_client, typescript)
        .map_err(|e| anyhow!("get live cell fail: {}", e))?
        .ok_or_else(|| anyhow!("eth header cell not exist"))?;

    let (un_confirmed_headers, _) = parse_main_chain_headers(cell.output_data.as_bytes().to_vec())
        .map_err(|e| anyhow!("parse header data fail: {}", e))?;

    let best_header = un_confirmed_headers
        .last()
        .ok_or_else(|| anyhow!("header is none"))?;
    let eth_light_client_height = best_header
        .number
        .ok_or_else(|| anyhow!("header number is none"))?;

    let mut msg = format!("ckb light client height : {:?}   ckb current height : {:?}   eth light client height : {:?}   eth current height : {:?}  ",ckb_light_client_height,ckb_current_height,eth_light_client_height,eth_current_height);

    if ckb_light_client_height + ckb_alarm_number < ckb_current_height {
        for conservator in ckb_conservator.iter() {
            msg = format!("{} @{} ", msg, conservator,);
        }
    }

    if eth_light_client_height.as_u64() + eth_alarm_number < eth_current_height.as_u64() {
        for conservator in eth_conservator.iter() {
            msg = format!("{} @{} ", msg, conservator,);
        }
    }

    let res = reqwest::get(format!("{}{:?}", alarm_url, msg).as_str())
        .await?
        .text()
        .await?;
    log::info!("{:?}", res);
    Ok(())
}
