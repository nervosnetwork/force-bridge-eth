use crate::dapp::db::server::CrosschainHistory;
use ckb_jsonrpc_types::TransactionView;
use ckb_jsonrpc_types::Uint128;
use serde::{Deserialize, Serialize};
use web3::types::{H160, U256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitTokenArgs {
    pub token_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockArgs {
    pub sender: String,
    pub token_address: String,
    pub amount: Uint128,
    pub bridge_fee: Uint128,
    pub ckb_recipient_address: String,
    pub replay_resist_outpoint: Option<String>,
    pub sudt_extra_data: String,
    pub gas_price: Uint128,
    pub nonce: Uint128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockResult {
    pub nonce: U256,
    pub to: Option<H160>,
    pub value: U256,
    pub gas_price: U256,
    pub gas: U256,
    pub data: String,
    pub raw: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurnArgs {
    pub from_lockscript_addr: String,
    pub tx_fee: Option<String>,
    pub unlock_fee: Uint128,
    pub amount: Uint128,
    pub token_address: String,
    pub recipient_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurnResult {
    pub raw_tx: TransactionView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetEthToCkbStatusArgs {
    pub eth_lock_tx_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetEthToCkbStatusResponse {
    pub eth_lock_tx_hash: String,
    pub status: String,
    pub err_msg: String,
    pub token_addr: Option<String>,
    pub sender_addr: Option<String>,
    pub locked_amount: Option<String>,
    pub bridge_fee: Option<String>,
    pub ckb_recipient_lockscript: Option<String>,
    pub sudt_extra_data: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub block_number: u64,
    pub replay_resist_outpoint: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetCkbToEthStatusArgs {
    pub ckb_burn_tx_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GetCkbToEthStatusResponse {
    pub id: u64,
    pub ckb_burn_tx_hash: String,
    pub status: String,
    pub recipient_addr: Option<String>,
    pub token_addr: Option<String>,
    pub token_amount: Option<String>,
    pub fee: Option<String>,
    pub eth_tx_hash: Option<String>,
    pub ckb_block_number: u64,
    pub eth_block_number: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetCrosschainHistoryArgs {
    pub ckb_recipient_lockscript_addr: Option<String>,
    pub ckb_recipient_lockscript: Option<String>,
    pub eth_recipient_addr: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GetCrosschainHistoryRes {
    pub eth_to_ckb: Vec<CrosschainHistory>,
    pub ckb_to_eth: Vec<CrosschainHistory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSudtBalanceArgs {
    pub address: Option<String>,
    pub script: Option<String>,
    pub token_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetBestBlockHeightArgs {
    pub chain: String,
}
