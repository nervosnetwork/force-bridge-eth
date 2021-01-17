use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::Done;
use super::indexer::{EthToCkbRecord, CkbToEthRecord};

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
    pub outpoint: String
}

pub async fn get_replay_resist_cells(pool: &MySqlPool, token: &String) -> Result<Vec<ReplayResistCell>> {
    let sql = r#"
SELECT id, outpoint FROM replay_resist_cells
WHERE token = ? AND status = ?
    "#;
    let cells = sqlx::query_as::<_, ReplayResistCell>(sql)
        .bind(token)
        .bind("available")
        .fetch_all(pool)
        .await?;
    Ok(cells)
}

pub async fn use_replay_resist_cell(pool: &MySqlPool, token: &String) -> Result<(usize, String)> {
    let outpoints = get_replay_resist_cells(pool, token).await?;
    if outpoints.len() == 0 {
        return Ok((0, "".to_string()))
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

pub async fn get_eth_to_ckb_status(
    pool: &MySqlPool,
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
