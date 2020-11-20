use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSudtBalanceArgs {
    pub address: String,
    pub token_address: String,
    pub lock_contract_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockArgs {
    pub lock_contract_address: String,
    pub token_address: String,
    pub amount: u128,
    pub bridge_fee: u128,
    pub ckb_recipient_address: String,
    pub replay_resist_outpoint: String,
    pub sudt_extra_data: String,
    pub gas_price: u128,
    pub nonce: u128,
}
