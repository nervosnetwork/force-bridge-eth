use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EthToCkbRecord {
    pub id: u64,
    pub eth_lock_tx_hash: String,
    pub status: String,
    pub token_addr: String,
    pub sender_addr: String,
    pub locked_amount: String,
    pub bridge_fee: String,
    pub ckb_recipient_lockscript: String,
    pub sudt_extra_data: Option<String>,
    pub ckb_tx_hash: Option<String>,
    pub eth_spv_proof: Option<String>,
    pub eth_block_number: u64,
    pub replay_resist_outpoint: String,
    pub ckb_block_number: u64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CrossChainHeightInfo {
    pub id: u8,
    pub height: u64,
    pub client_height: u64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EthUnConfirmedBlock {
    pub id: u64,
    pub number: u64,
    pub hash: String,
}

pub async fn get_eth_unconfirmed_block(
    pool: &MySqlPool,
    id: u64,
) -> Result<Option<EthUnConfirmedBlock>> {
    let sql = r#"select * from eth_unconfirmed_block where id = ?"#;
    let ret = sqlx::query_as::<_, EthUnConfirmedBlock>(sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(ret)
}

pub async fn get_max_eth_unconfirmed_block(
    pool: &MySqlPool,
) -> Result<Option<EthUnConfirmedBlock>> {
    let sql = r#"select * from eth_unconfirmed_block order by number desc limit 1"#;
    let ret = sqlx::query_as::<_, EthUnConfirmedBlock>(sql)
        .fetch_optional(pool)
        .await?;
    Ok(ret)
}

pub async fn get_eth_unconfirmed_blocks(pool: &MySqlPool) -> Result<Vec<EthUnConfirmedBlock>> {
    let sql = r#"select * from eth_unconfirmed_block order by number"#;
    let ret = sqlx::query_as::<_, EthUnConfirmedBlock>(sql)
        .fetch_all(pool)
        .await?;
    Ok(ret)
}

pub async fn insert_eth_unconfirmed_blocks(
    pool: &MySqlPool,
    records: &[EthUnConfirmedBlock],
) -> Result<()> {
    let mut sql = String::from(
        r"
INSERT INTO eth_unconfirmed_block (id, number, hash)
VALUES ",
    );
    for _ in records {
        sql = format!("{}{}", sql, "(?,?,?),");
    }
    let len = sql.len() - 1;
    let mut ret = sqlx::query(&sql[..len]);
    for record in records {
        ret = ret
            .bind(record.id)
            .bind(record.number)
            .bind(record.hash.clone())
    }
    ret.execute(pool).await?;
    Ok(())
}

pub async fn insert_eth_unconfirmed_block(
    pool: &mut Transaction<'_, MySql>,
    record: &EthUnConfirmedBlock,
) -> Result<()> {
    let sql = r#"insert into eth_unconfirmed_block(id, number, hash)
    values(?,?,?)"#;
    sqlx::query(sql)
        .bind(record.id)
        .bind(record.number)
        .bind(record.hash.clone())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_eth_unconfirmed_block(
    pool: &mut Transaction<'_, MySql>,
    record: &EthUnConfirmedBlock,
) -> Result<()> {
    let sql = r#"update eth_unconfirmed_block set
    number = ?, hash = ? WHERE id = ?"#;
    sqlx::query(sql)
        .bind(record.number)
        .bind(record.hash.clone())
        .bind(record.id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_eth_unconfirmed_block(
    pool: &mut Transaction<'_, MySql>,
    number: u64,
) -> Result<()> {
    let sql = r"delete from eth_unconfirmed_block where number >= ?";
    sqlx::query(sql).bind(number).execute(pool).await?;
    Ok(())
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CkbUnConfirmedBlock {
    pub id: u64,
    pub number: u64,
    pub hash: String,
}

pub async fn get_ckb_unconfirmed_block(
    pool: &MySqlPool,
    id: u64,
) -> Result<Option<CkbUnConfirmedBlock>> {
    let sql = r#"select * from ckb_unconfirmed_block where id = ?"#;
    let ret = sqlx::query_as::<_, CkbUnConfirmedBlock>(sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(ret)
}

pub async fn get_max_ckb_unconfirmed_block(
    pool: &MySqlPool,
) -> Result<Option<CkbUnConfirmedBlock>> {
    let sql = r#"select * from ckb_unconfirmed_block order by number desc limit 1"#;
    let ret = sqlx::query_as::<_, CkbUnConfirmedBlock>(sql)
        .fetch_optional(pool)
        .await?;
    Ok(ret)
}

pub async fn get_ckb_unconfirmed_blocks(pool: &MySqlPool) -> Result<Vec<CkbUnConfirmedBlock>> {
    let sql = r#"select * from ckb_unconfirmed_block order by number"#;
    let ret = sqlx::query_as::<_, CkbUnConfirmedBlock>(sql)
        .fetch_all(pool)
        .await?;
    Ok(ret)
}

pub async fn insert_ckb_unconfirmed_blocks(
    pool: &MySqlPool,
    records: &[CkbUnConfirmedBlock],
) -> Result<()> {
    let mut sql = String::from(
        r"
INSERT INTO ckb_unconfirmed_block (id, number, hash)
VALUES ",
    );
    for _ in records {
        sql = format!("{}{}", sql, "(?,?,?),");
    }
    let len = sql.len() - 1;
    let mut ret = sqlx::query(&sql[..len]);
    for record in records {
        ret = ret
            .bind(record.id)
            .bind(record.number)
            .bind(record.hash.clone())
    }
    ret.execute(pool).await?;
    Ok(())
}

pub async fn delete_ckb_unconfirmed_block(
    pool: &mut Transaction<'_, MySql>,
    number: u64,
) -> Result<()> {
    let sql = r"delete from ckb_unconfirmed_block where number >= ?";
    sqlx::query(sql).bind(number).execute(pool).await?;
    Ok(())
}

pub async fn update_ckb_unconfirmed_block(
    pool: &mut Transaction<'_, MySql>,
    record: &CkbUnConfirmedBlock,
) -> Result<()> {
    let sql = r#"update ckb_unconfirmed_block set
    number = ?, hash = ? WHERE id = ?"#;
    sqlx::query(sql)
        .bind(record.number)
        .bind(record.hash.clone())
        .bind(record.id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_ckb_unconfirmed_block(
    pool: &mut Transaction<'_, MySql>,
    record: &CkbUnConfirmedBlock,
) -> Result<()> {
    let sql = r#"insert into ckb_unconfirmed_block(id, number, hash)
    values(?,?,?)"#;
    sqlx::query(sql)
        .bind(record.id)
        .bind(record.number)
        .bind(record.hash.clone())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_ckb_to_eth_records(
    pool: &mut Transaction<'_, MySql>,
    number: u64,
) -> Result<()> {
    let sql = r"delete from ckb_to_eth where ckb_block_number >= ?";
    sqlx::query(sql).bind(number).execute(pool).await?;
    Ok(())
}

pub async fn reset_eth_to_ckb_record_status(
    pool: &mut Transaction<'_, MySql>,
    number: u64,
) -> Result<()> {
    let sql = r#"update eth_to_ckb set status = 'pending' where ckb_block_number >= ? and status = 'success'"#;
    sqlx::query(sql).bind(number).execute(pool).await?;
    Ok(())
}

pub async fn get_height_info(pool: &MySqlPool, id: u8) -> Result<CrossChainHeightInfo> {
    let sql = r#"select * from cross_chain_height_info where id = ?"#;
    let ret = sqlx::query_as::<_, CrossChainHeightInfo>(sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("the record is not exist"))?;
    Ok(ret)
}

pub async fn update_cross_chain_height_info(
    pool: &mut Transaction<'_, MySql>,
    info: &CrossChainHeightInfo,
) -> Result<()> {
    let sql = r#"update cross_chain_height_info set
    height = ?, client_height = ? WHERE id = ?"#;
    sqlx::query(sql)
        .bind(info.height)
        .bind(info.client_height)
        .bind(info.id)
        .execute(pool)
        .await?;
    Ok(())
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

pub async fn get_eth_to_ckb_record_by_outpoint(
    pool: &MySqlPool,
    outpoint: String,
) -> Result<Option<EthToCkbRecord>> {
    let sql = r#"SELECT *
FROM eth_to_ckb
where replay_resist_outpoint = ?"#;
    let ret = sqlx::query_as::<_, EthToCkbRecord>(sql)
        .bind(outpoint)
        .fetch_optional(pool)
        .await?;
    Ok(ret)
}

pub async fn is_eth_to_ckb_record_exist(pool: &MySqlPool, eth_tx_hash: &str) -> Result<bool> {
    let sql = r#"
SELECT *
FROM eth_to_ckb
where eth_lock_tx_hash = ?
        "#;
    let ret = sqlx::query_as::<_, EthToCkbRecord>(sql)
        .bind(eth_tx_hash)
        .fetch_all(pool)
        .await?;
    Ok(!ret.is_empty())
}

pub async fn update_eth_to_ckb_status(
    pool: &mut Transaction<'_, MySql>,
    record: &EthToCkbRecord,
) -> Result<()> {
    let sql =
        r#"UPDATE eth_to_ckb SET status = ?, ckb_block_number = ?, ckb_tx_hash = ? WHERE id = ?"#;
    sqlx::query(sql)
        .bind(record.status.clone())
        .bind(record.ckb_block_number)
        .bind(record.ckb_tx_hash.clone())
        .bind(record.id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_eth_to_ckb_records(
    pool: &mut Transaction<'_, MySql>,
    number: u64,
) -> Result<()> {
    let sql = r"delete from eth_to_ckb where eth_block_number >= ?";
    sqlx::query(sql).bind(number).execute(pool).await?;
    Ok(())
}

pub async fn create_eth_to_ckb_record(
    pool: &mut Transaction<'_, MySql>,
    records: &[EthToCkbRecord],
) -> Result<()> {
    let mut sql = String::from(
        r"
INSERT INTO eth_to_ckb ( eth_lock_tx_hash, status, token_addr, sender_addr, locked_amount, bridge_fee, 
ckb_recipient_lockscript, sudt_extra_data, ckb_tx_hash, eth_spv_proof, eth_block_number, replay_resist_outpoint)
VALUES ",
    );
    for _ in records {
        sql = format!("{}{}", sql, "( ?,?,?,?,?,?,?,?,?,?,?,?),");
    }
    let len = sql.len() - 1;
    let mut ret = sqlx::query(&sql[..len]);
    for record in records {
        ret = ret
            .bind(record.eth_lock_tx_hash.clone())
            .bind(record.status.clone())
            .bind(record.token_addr.clone())
            .bind(record.sender_addr.clone())
            .bind(record.locked_amount.clone())
            .bind(record.bridge_fee.clone())
            .bind(record.ckb_recipient_lockscript.clone())
            .bind(record.sudt_extra_data.as_ref())
            .bind(record.ckb_tx_hash.as_ref())
            .bind(record.eth_spv_proof.as_ref())
            .bind(record.eth_block_number)
            .bind(record.replay_resist_outpoint.clone());
    }
    ret.execute(pool).await?;
    Ok(())
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CkbToEthRecord {
    pub id: u64,
    pub ckb_burn_tx_hash: String,
    pub status: String,
    pub recipient_addr: String,
    pub token_addr: String,
    pub token_amount: String,
    pub fee: String,
    pub eth_tx_hash: Option<String>,
    pub ckb_spv_proof: Option<String>,
    pub ckb_block_number: u64,
    pub ckb_raw_tx: String,
    pub eth_block_number: u64,
    pub bridge_lock_hash: String,
    pub lock_contract_addr: String,
}

pub async fn get_latest_ckb_to_eth_record(pool: &MySqlPool) -> Result<Option<CkbToEthRecord>> {
    Ok(sqlx::query_as::<_, CkbToEthRecord>(
        r#"
SELECT *
FROM ckb_to_eth
order by id desc limit 1
        "#,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create_ckb_to_eth_record(
    pool: &mut Transaction<'_, MySql>,
    records: &[CkbToEthRecord],
) -> Result<()> {
    let mut sql = String::from(
        r"
INSERT INTO ckb_to_eth ( ckb_burn_tx_hash, status, recipient_addr, token_addr, token_amount, fee, 
eth_tx_hash, ckb_spv_proof, ckb_block_number, ckb_raw_tx, lock_contract_addr, bridge_lock_hash)
VALUES ",
    );
    for _ in records {
        sql = format!("{}{}", sql, "( ?,?,?,?,?,?,?,?,?,?,?,?),");
    }
    let len = sql.len() - 1;
    let mut ret = sqlx::query(&sql[..len]);
    for record in records {
        ret = ret
            .bind(record.ckb_burn_tx_hash.clone())
            .bind(record.status.clone())
            .bind(record.recipient_addr.clone())
            .bind(record.token_addr.clone())
            .bind(record.token_amount.clone())
            .bind(record.fee.clone())
            .bind(record.eth_tx_hash.as_ref())
            .bind(record.ckb_spv_proof.as_ref())
            .bind(record.ckb_block_number)
            .bind(record.ckb_raw_tx.clone())
            .bind(record.lock_contract_addr.clone())
            .bind(record.bridge_lock_hash.clone())
    }
    ret.execute(pool).await?;
    Ok(())
}

pub async fn is_ckb_to_eth_record_exist(pool: &MySqlPool, ckb_tx_hash: &str) -> Result<bool> {
    let sql = r#"
SELECT *
FROM ckb_to_eth
where ckb_burn_tx_hash = ?
        "#;
    let ret = sqlx::query_as::<_, CkbToEthRecord>(sql)
        .bind(ckb_tx_hash)
        .fetch_all(pool)
        .await?;
    Ok(!ret.is_empty())
}

pub async fn get_ckb_to_eth_record_by_eth_hash(
    pool: &MySqlPool,
    hash: String,
) -> Result<Option<CkbToEthRecord>> {
    let sql = r#"
SELECT *
FROM ckb_to_eth where eth_tx_hash = ?
        "#;

    Ok(sqlx::query_as::<_, CkbToEthRecord>(sql)
        .bind(hash)
        .fetch_optional(pool)
        .await?)
}

pub async fn reset_ckb_to_eth_record_status(
    pool: &mut Transaction<'_, MySql>,
    number: u64,
) -> Result<()> {
    let sql = r#"update ckb_to_eth set status = 'pending' where eth_block_number >= ? and status = 'success'"#;
    sqlx::query(sql).bind(number).execute(pool).await?;
    Ok(())
}

pub async fn update_ckb_to_eth_record_status(
    pool: &mut Transaction<'_, MySql>,
    ckb_tx_hash: String,
    eth_tx_hash: String,
    status: &str,
    eth_block_number: u64,
) -> Result<()> {
    let sql = r#"
UPDATE ckb_to_eth SET
    status = ?,
    eth_tx_hash = ?,
    eth_block_number = ? 
WHERE  ckb_burn_tx_hash = ?
        "#;
    sqlx::query(sql)
        .bind(status)
        .bind(eth_tx_hash)
        .bind(eth_block_number)
        .bind(ckb_tx_hash)
        .execute(pool)
        .await?;
    Ok(())
}
