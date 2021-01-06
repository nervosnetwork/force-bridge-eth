use anyhow::Result;
use sqlx::mysql::MySqlPool;

#[derive(sqlx::FromRow, Debug)]
pub struct MintTask {
    pub lock_tx_hash: String,
    pub lock_tx_proof: String,
    pub block_number: u64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct BlockNumber {
    pub inner: u64,
}

pub async fn last_relayed_number(pool: &MySqlPool) -> Result<u64> {
    let sql = r#"
SELECT block_number FROM eth_tx_relayer order by block_number desc limit 1
    "#;
    let block_number = sqlx::query_as::<_, BlockNumber>(sql)
        .fetch_optional(pool)
        .await?;
    Ok(block_number.map_or(0, |v| v.inner))
}

pub async fn get_mint_tasks(
    pool: &MySqlPool,
    start_block: u64,
    end_block: u64,
) -> Result<Vec<MintTask>> {
    let sql = r#"
SELECT eth_lock_tx_hash as lock_tx_hash, eth_spv_proof as lock_tx_proof, block_number
FROM eth_to_ckb
WHERE status = ? AND block_number BETWEEN ? AND ?
    "#;
    let tasks = sqlx::query_as::<_, MintTask>(sql)
        .bind("pending")
        .bind(start_block)
        .bind(end_block)
        .fetch_all(pool)
        .await?;
    Ok(tasks)
}

pub async fn get_retry_tasks(pool: &MySqlPool) -> Result<Vec<MintTask>> {
    let sql = r#"
SELECT block_number, lock_tx_hash, lock_tx_proof
FROM eth_tx_relayer
WHERE status = ?
    "#;
    let tasks = sqlx::query_as::<_, MintTask>(sql)
        .bind("retryable")
        .fetch_all(pool)
        .await?;
    Ok(tasks)
}

pub async fn store_mint_task(pool: &MySqlPool, task: MintTask) -> Result<()> {
    let sql = r#"
INSERT INTO eth_tx_relayer (block_number, lock_tx_hash, lock_tx_proof)
VALUES (?,?,?)
    "#;
    sqlx::query(sql)
        .bind(task.block_number)
        .bind(task.lock_tx_hash)
        .bind(task.lock_tx_proof)
        .execute(pool)
        .await?;
    Ok(())
}


#[cfg(test)]
mod test {
    use super::last_relayed_number;
    use sqlx::MySqlPool;

    #[tokio::test]
    async fn get_latest_number() {
        println!("ok");
        let pool = MySqlPool::connect("mysql://root:root1234@127.0.0.1:3306/forcedb")
            .await
            .expect("connect db error");
        let number = last_relayed_number(&pool).await.expect("get number error");
        println!("number: {:?}", number);
    }
}
