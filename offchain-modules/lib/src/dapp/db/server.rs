use super::indexer::{CkbToEthRecord, EthToCkbRecord};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CrosschainHistory {
    pub id: u64,
    pub eth_tx_hash: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub status: String,
    pub sort: String,
    pub amount: String,
    pub token_addr: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct ReplayResistCell {
    pub id: u64,
    pub outpoint: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct RelayStatus {
    pub status: String,
    pub err_msg: String,
}

pub async fn delete_replay_resist_cells(pool: &MySqlPool, cells: &[u64]) -> Result<()> {
    let mut tx = pool.begin().await?;
    let sql = r#"
DELETE FROM replay_resist_cells
WHERE id = ?
    "#;
    for id in cells.iter() {
        sqlx::query(sql).bind(id).execute(&mut tx).await?;
    }
    tx.commit()
        .await
        .map_err(|e| anyhow!("commit batch delete cells error: {:?}", e))
}

pub async fn add_replay_resist_cells(
    pool: &MySqlPool,
    cells: &[String],
    token: &str,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    let sql = r#"
INSERT INTO replay_resist_cells (token, outpoint, status)
VALUES (?,?,?)
    "#;
    for outpoint in cells.iter() {
        sqlx::query(sql)
            .bind(token.to_string())
            .bind(outpoint.clone())
            .bind("available")
            .execute(&mut tx)
            .await?;
    }
    tx.commit()
        .await
        .map_err(|e| anyhow!("commit batch insert replay resist cells error: {:?}", e))
}

pub async fn get_replay_resist_cells(
    pool: &MySqlPool,
    token: &str,
    status: &str,
) -> Result<Vec<ReplayResistCell>> {
    let sql = r#"
SELECT id, outpoint FROM replay_resist_cells
WHERE token = ? AND status = ?
    "#;
    let cells = sqlx::query_as::<_, ReplayResistCell>(sql)
        .bind(token)
        .bind(status)
        .fetch_all(pool)
        .await?;
    Ok(cells)
}

pub async fn use_replay_resist_cell(pool: &MySqlPool, token: &str) -> Result<(usize, String)> {
    let outpoints = get_replay_resist_cells(pool, token, "available").await?;
    if outpoints.is_empty() {
        return Ok((0, "".to_string()));
    }

    let sql = r#"
UPDATE replay_resist_cells
SET status = ?
WHERE id = ?
    "#;
    let outpoint = outpoints[0].clone();
    sqlx::query(sql)
        .bind("used")
        .bind(outpoint.id)
        .execute(pool)
        .await?;
    Ok((outpoints.len(), outpoint.outpoint))
}

pub async fn get_eth_to_ckb_indexer_status(
    pool: &MySqlPool,
    eth_lock_tx_hash: &str,
) -> Result<Option<EthToCkbRecord>> {
    let ret = sqlx::query_as::<_, EthToCkbRecord>(
        r#"
SELECT *
FROM eth_to_ckb
where eth_lock_tx_hash = ?
        "#,
    )
    .bind(eth_lock_tx_hash)
    .fetch_optional(pool)
    .await?;
    Ok(ret)
}

pub async fn get_eth_to_ckb_relay_status(
    pool: &MySqlPool,
    eth_lock_tx_hash: &str,
) -> Result<Option<RelayStatus>> {
    let ret = sqlx::query_as::<_, RelayStatus>(
        r#"
SELECT status, err_msg
FROM eth_tx_relayer
where lock_tx_hash = ?
        "#,
    )
    .bind(eth_lock_tx_hash)
    .fetch_optional(pool)
    .await?;
    Ok(ret)
}

pub async fn get_ckb_to_eth_crosschain_history(
    pool: &MySqlPool,
    eth_recipient_address: &str,
) -> Result<Vec<CrosschainHistory>> {
    Ok(sqlx::query_as::<_, CrosschainHistory>(
        r#"
SELECT id, eth_tx_hash, ckb_burn_tx_hash as ckb_tx_hash, status, 'ckb_to_eth' as sort, token_amount as amount, token_addr
FROM ckb_to_eth
where recipient_addr = ?
        "#,
    )
        .bind(eth_recipient_address)
        .fetch_all(pool)
        .await?)
}

pub async fn get_eth_to_ckb_crosschain_history(
    pool: &MySqlPool,
    ckb_recipient_lockscript: &str,
) -> Result<Vec<CrosschainHistory>> {
    Ok(sqlx::query_as::<_, CrosschainHistory>(
        r#"
SELECT id, eth_lock_tx_hash as eth_tx_hash, ckb_tx_hash, status, 'eth_to_ckb' as sort, locked_amount as amount, token_addr
FROM eth_to_ckb
where ckb_recipient_lockscript = ?
        "#,
    )
        .bind(ckb_recipient_lockscript)
        .fetch_all(pool)
        .await?)
}

pub async fn get_ckb_to_eth_status(
    pool: &MySqlPool,
    ckb_burn_tx_hash: &str,
) -> Result<Option<CkbToEthRecord>> {
    Ok(sqlx::query_as::<_, CkbToEthRecord>(
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
