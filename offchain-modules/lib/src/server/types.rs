use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBridgeCellArgs {
    pub recipient_address: String,
    pub eth_token_address: String,
    pub bridge_fee: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBridgeCellResponse {
    pub outpoint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurnArgs {
    pub from_lockscript_addr: String,
    pub tx_fee: u64,
    pub unlock_fee: u128,
    pub amount: u128,
    pub token_address: String,
    pub lock_contract_address: String,
    pub recipient_address: String,
}
