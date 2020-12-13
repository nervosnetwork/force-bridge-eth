use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::Done;

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EthToCkbRecord {
    pub id: i64,
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
}

pub async fn create_eth_to_ckb_status_record(pool: &SqlitePool, tx_hash: String) -> Result<i64> {
    let id = sqlx::query(
        r#"
INSERT INTO eth_to_ckb ( eth_lock_tx_hash )
VALUES ( ?1 )
        "#,
    )
    .bind(tx_hash)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_eth_to_ckb_status(pool: &SqlitePool, record: &EthToCkbRecord) -> Result<bool> {
    log::info!("update_eth_to_ckb_status, record: {:?}", record);
    let rows_affected = sqlx::query(
        r#"
UPDATE eth_to_ckb SET 
    status = ?2,
    token_addr = ?3,
    sender_addr = ?4,
    locked_amount = ?5,
    bridge_fee = ?6,
    ckb_recipient_lockscript = ?7,
    sudt_extra_data = ?8,
    ckb_tx_hash = ?9,
    err_msg = ?10
WHERE id = ?1
        "#,
    )
    .bind(record.id)
    .bind(record.status.clone())
    .bind(record.token_addr.as_ref())
    .bind(record.sender_addr.as_ref())
    .bind(record.locked_amount.as_ref())
    .bind(record.bridge_fee.as_ref())
    .bind(record.ckb_recipient_lockscript.as_ref())
    .bind(record.sudt_extra_data.as_ref())
    .bind(record.ckb_tx_hash.as_ref())
    .bind(record.err_msg.as_ref())
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

pub async fn get_eth_to_ckb_status(
    pool: &SqlitePool,
    eth_lock_tx_hash: &str,
) -> Result<Option<EthToCkbRecord>> {
    Ok(sqlx::query_as::<_, EthToCkbRecord>(
        r#"
SELECT *
FROM eth_to_ckb
where eth_lock_tx_hash = ?
        "#,
    )
    .bind(eth_lock_tx_hash)
    .fetch_optional(pool)
    .await?)
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CrosschainHistory {
    pub id: i64,
    pub eth_tx_hash: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub status: String,
    pub sort: String,
    pub amount: String,
}

pub async fn get_ckb_to_eth_crosschain_history(
    pool: &SqlitePool,
    eth_recipient_address: &str,
) -> Result<Vec<CrosschainHistory>> {
    Ok(sqlx::query_as::<_, CrosschainHistory>(
        r#"
SELECT id, eth_tx_hash, ckb_burn_tx_hash as ckb_tx_hash, status, 'ckb_to_eth' as sort, token_amount as amount
FROM eth_to_ckb
where recipient_addr = ?1
        "#,
    )
        .bind(eth_recipient_address)
        .fetch_all(pool)
        .await?)
}

pub async fn get_eth_to_ckb_crosschain_history(
    pool: &SqlitePool,
    ckb_recipient_lockscript: &str,
) -> Result<Vec<CrosschainHistory>> {
    Ok(sqlx::query_as::<_, CrosschainHistory>(
        r#"
SELECT id, eth_lock_tx_hash as eth_tx_hash, ckb_tx_hash, status, 'eth_to_ckb' as sort, locked_amount as amount
FROM eth_to_ckb
where ckb_recipient_lockscript = ?1
        "#,
    )
    .bind(ckb_recipient_lockscript)
    .fetch_all(pool)
    .await?)
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
}

pub async fn create_ckb_to_eth_status_record(pool: &SqlitePool, tx_hash: String) -> Result<i64> {
    let id = sqlx::query(
        r#"
INSERT INTO ckb_to_eth ( ckb_burn_tx_hash )
VALUES ( ?1 )
        "#,
    )
    .bind(tx_hash)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_ckb_to_eth_status(pool: &SqlitePool, record: &CkbToEthRecord) -> Result<bool> {
    log::info!("update_ckb_to_eth_status, record: {:?}", record);
    let rows_affected = sqlx::query(
        r#"
UPDATE ckb_to_eth SET 
    status = ?2,
    recipient_addr = ?3,
    token_addr = ?4,
    token_amount = ?5,
    fee = ?6,
    eth_tx_hash = ?7,
    err_msg = ?8
WHERE id = ?1
        "#,
    )
    .bind(record.id)
    .bind(record.status.clone())
    .bind(record.recipient_addr.as_ref())
    .bind(record.token_addr.as_ref())
    .bind(record.token_amount.as_ref())
    .bind(record.fee.as_ref())
    .bind(record.eth_tx_hash.as_ref())
    .bind(record.err_msg.as_ref())
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

pub async fn get_ckb_to_eth_status(
    pool: &SqlitePool,
    ckb_burn_tx_hash: &str,
) -> Result<Option<EthToCkbRecord>> {
    Ok(sqlx::query_as::<_, EthToCkbRecord>(
        r#"
SELECT *
FROM ckb_to_eth
where ckb_burn_tx_hash = ?
        "#,
    )
    .bind(ckb_burn_tx_hash)
    .fetch_optional(pool)
    .await?)
}
