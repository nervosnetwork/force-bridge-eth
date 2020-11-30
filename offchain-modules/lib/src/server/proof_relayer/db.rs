use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::Done;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
    let mut conn = pool.acquire().await?;
    let id = sqlx::query!(
        r#"
INSERT INTO eth_to_ckb ( eth_lock_tx_hash )
VALUES ( ?1 )
        "#,
        tx_hash
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_eth_to_ckb_status(pool: &SqlitePool, record: &EthToCkbRecord) -> Result<bool> {
    log::info!("update_eth_to_ckb_status, record: {:?}", record);
    let rows_affected = sqlx::query!(
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
        record.id,
        record.status,
        record.token_addr,
        record.sender_addr,
        record.locked_amount,
        record.bridge_fee,
        record.ckb_recipient_lockscript,
        record.sudt_extra_data,
        record.ckb_tx_hash,
        record.err_msg
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

pub async fn get_eth_to_ckb_status(
    pool: &SqlitePool,
    eth_lock_tx_hash: &str,
) -> Result<Option<EthToCkbRecord>> {
    Ok(sqlx::query_as!(
        EthToCkbRecord,
        r#"
SELECT *
FROM eth_to_ckb
where eth_lock_tx_hash = ?
        "#,
        eth_lock_tx_hash
    )
    .fetch_optional(pool)
    .await?)
}
