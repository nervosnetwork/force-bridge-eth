use super::errors::RpcError;
use crate::dapp::db::server::CrosschainHistory;
use ckb_jsonrpc_types::Uint128;
use ckb_jsonrpc_types::{Script as ScriptJson, TransactionView};
use ckb_types::packed::Script;
use ckb_types::prelude::Entity;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
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
pub struct RecycleRecipientCellArgs {
    pub from_lockscript_addr: String,
    pub tx_fee: Option<String>,
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
    pub token_addr: String,
    pub sender_addr: String,
    pub locked_amount: String,
    pub bridge_fee: String,
    pub ckb_recipient_lockscript: String,
    pub sudt_extra_data: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub block_number: u64,
    pub replay_resist_outpoint: String,
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
    pub recipient_addr: String,
    pub token_addr: String,
    pub token_amount: String,
    pub fee: String,
    pub eth_tx_hash: Option<String>,
    pub ckb_block_number: u64,
    pub eth_block_number: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetCrosschainHistoryArgs {
    pub lock_sender_addr: Option<String>,
    pub eth_recipient_addr: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GetCrosschainHistoryRes {
    pub eth_to_ckb: Vec<EthToCkbCrosschainHistoryRes>,
    pub ckb_to_eth: Vec<CkbToEthCrosschainHistoryRes>,
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EthToCkbCrosschainHistoryRes {
    pub id: u64,
    pub eth_tx_hash: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub status: String,
    pub sort: String,
    pub amount: String,
    pub token_addr: String,
    pub recipient_lockscript: ScriptJson,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CkbToEthCrosschainHistoryRes {
    pub id: u64,
    pub eth_tx_hash: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub status: String,
    pub sort: String,
    pub amount: String,
    pub token_addr: String,
    pub recipient_addr: String,
}

impl TryFrom<CrosschainHistory> for EthToCkbCrosschainHistoryRes {
    type Error = RpcError;

    fn try_from(history: CrosschainHistory) -> Result<Self, Self::Error> {
        let recipient_addr = hex::decode(history.recipient_addr).map_err(|e| {
            RpcError::ServerError(format!(
                "hex decode crosschain history recipient addr error: {:?}",
                e
            ))
        })?;
        let recipient_lockscript = Script::from_slice(recipient_addr.as_slice()).map_err(|e| {
            RpcError::ServerError(format!(
                "molecule decode crosschain history recipient lockscript error: {:?}",
                e
            ))
        })?;
        let recipient_lockscript: ScriptJson = recipient_lockscript.into();
        Ok(Self {
            id: history.id,
            eth_tx_hash: history.eth_tx_hash.as_ref().map(|h| pad_0x_prefix(&h)),
            ckb_tx_hash: history.ckb_tx_hash.as_ref().map(|h| pad_0x_prefix(&h)),
            status: history.status,
            sort: history.sort,
            amount: history.amount,
            token_addr: pad_0x_prefix(&history.token_addr),
            recipient_lockscript,
        })
    }
}

impl From<CrosschainHistory> for CkbToEthCrosschainHistoryRes {
    fn from(history: CrosschainHistory) -> Self {
        Self {
            id: history.id,
            eth_tx_hash: history.eth_tx_hash.as_ref().map(|h| pad_0x_prefix(&h)),
            ckb_tx_hash: history.ckb_tx_hash.as_ref().map(|h| pad_0x_prefix(&h)),
            status: history.status,
            sort: history.sort,
            amount: history.amount,
            token_addr: pad_0x_prefix(&history.token_addr),
            recipient_addr: pad_0x_prefix(&history.recipient_addr),
        }
    }
}

pub fn pad_0x_prefix(s: &str) -> String {
    if !s.starts_with("0x") && !s.starts_with("0X") {
        let mut res = "0x".to_owned();
        res.push_str(s);
        res
    } else {
        s.to_owned()
    }
}
