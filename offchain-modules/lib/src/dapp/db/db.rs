use crate::dapp::server::proof_relayer::db::CrosschainHistory;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EthToCkbRecord {
    pub id: u64,
    pub eth_lock_tx_hash: String,
    pub status: String,
    pub token_addr: Option<String>,
    pub sender_addr: Option<String>,
    pub locked_amount: Option<String>,
    pub bridge_fee: Option<String>,
    pub ckb_recipient_lockscript: Option<String>,
    pub sudt_extra_data: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub err_msg: Option<String>,
    pub eth_spv_proof: Option<Vec<u8>>,
}

pub async fn get_latest_eth_to_ckb_record(pool: &MySqlPool) -> Result<Option<EthToCkbRecord>> {
    Ok(sqlx::query_as::<_, EthToCkbRecord>(
        r#"
SELECT *
FROM eth_to_ckb
order by id desc limit 1
        "#,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn is_eth_to_ckb_record_exist(pool: &MySqlPool, eth_tx_hash: &str) -> Result<bool> {
    let sql = r#"
SELECT id, eth_lock_tx_hash as eth_tx_hash, ckb_tx_hash, status, 'eth_to_ckb' as sort, locked_amount as amount, token_addr
FROM eth_to_ckb
where eth_lock_tx_hash = ?
        "#;
    let ret = sqlx::query_as::<_, CrosschainHistory>(sql)
        .bind(eth_tx_hash)
        .fetch_all(pool)
        .await?;
    if ret.is_empty() {
        return Ok(false);
    }
    Ok(true)
}

pub async fn create_eth_to_ckb_record(pool: &MySqlPool, record: &EthToCkbRecord) -> Result<u64> {
    let sql = r#"
INSERT INTO eth_to_ckb ( eth_lock_tx_hash, status, token_addr, sender_addr, locked_amount, bridge_fee, 
ckb_recipient_lockscript, sudt_extra_data, ckb_tx_hash, err_msg, eth_spv_proof)
VALUES ( ?,?,?,?,?,?,?,?,?,?,?)"#;
    let id = sqlx::query(sql)
        .bind(record.eth_lock_tx_hash.clone())
        .bind(record.status.clone())
        .bind(record.token_addr.as_ref())
        .bind(record.sender_addr.as_ref())
        .bind(record.locked_amount.as_ref())
        .bind(record.bridge_fee.as_ref())
        .bind(record.ckb_recipient_lockscript.as_ref())
        .bind(record.sudt_extra_data.as_ref())
        .bind(record.ckb_tx_hash.as_ref())
        .bind(record.err_msg.as_ref())
        .bind(record.eth_spv_proof.as_ref())
        .execute(pool)
        .await?
        .last_insert_id();
    Ok(id)
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CkbToEthRecord {
    pub id: i64,
    pub ckb_burn_tx_hash: String,
    pub status: String,
    pub recipient_addr: Option<String>,
    pub token_addr: Option<String>,
    pub token_amount: Option<String>,
    pub fee: Option<String>,
    pub eth_tx_hash: Option<String>,
    pub err_msg: Option<String>,
    pub ckb_spv_proof: Option<Vec<u8>>,
}
