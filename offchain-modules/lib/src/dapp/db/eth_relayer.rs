use anyhow::{anyhow, Result};
use sqlx::mysql::MySqlPool;
use sqlx::Done;

#[derive(sqlx::FromRow, Debug)]
pub struct MintTask {
    pub lock_tx_hash: String,
    pub lock_tx_proof: String,
    pub block_number: u64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct BlockNumber {
    pub block_number: u64,
}

pub async fn last_relayed_number(pool: &MySqlPool) -> Result<u64> {
    let sql = r#"
SELECT block_number FROM eth_tx_relayer order by block_number desc limit 1
    "#;
    let block_number = sqlx::query_as::<_, BlockNumber>(sql)
        .fetch_optional(pool)
        .await?;
    Ok(block_number.map_or(0, |v| v.block_number))
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

pub async fn store_mint_tasks(pool: &MySqlPool, tasks: &[MintTask]) -> Result<()> {
    let mut tx = pool.begin().await?;
    let sql = r#"
INSERT INTO eth_tx_relayer (block_number, lock_tx_hash, lock_tx_proof)
VALUES (?,?,?)
    "#;
    for task in tasks.iter() {
        sqlx::query(sql)
            .bind(task.block_number)
            .bind(task.lock_tx_hash.clone())
            .bind(task.lock_tx_proof.clone())
            .execute(&mut tx)
            .await?;
    }
    tx.commit()
        .await
        .map_err(|e| anyhow!("commit batch insert mint tasks error: {:?}", e))
}

pub async fn update_relayed_tx(
    pool: &MySqlPool,
    lock_tx_hash: String,
    status: String,
    err_msg: String,
) -> Result<bool> {
    let sql = r#"
UPDATE eth_tx_relayer
SET status = ?, err_msg = ?
WHERE lock_tx_hash = ?
    "#;
    let rows_affected = sqlx::query(sql)
        .bind(status)
        .bind(err_msg)
        .bind(lock_tx_hash)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows_affected > 0)
}

pub async fn delete_relayed_tx(pool: &MySqlPool, lock_tx_hash: String) -> Result<bool> {
    let sql = r#"
DELETE FROM eth_tx_relayer
WHERE lock_tx_hash = ?
    "#;
    let rows_affected = sqlx::query(sql)
        .bind(lock_tx_hash)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows_affected > 0)
}
//
// #[cfg(test)]
// mod test {
//     use super::{
//         delete_relayed_tx, get_retry_tasks, last_relayed_number, store_mint_tasks,
//         update_relayed_tx, MintTask,
//     };
//     use sqlx::MySqlPool;
//
//     #[tokio::test]
//     async fn test_get_latest_number() {
//         let pool = MySqlPool::connect("mysql://root:root1234@127.0.0.1:3306/forcedb")
//             .await
//             .expect("connect db error");
//         let number = last_relayed_number(&pool).await.expect("get number error");
//         println!("number: {:?}", number);
//     }
//
//     #[tokio::test]
//     async fn test_store_mint_tasks() {
//         let pool = MySqlPool::connect("mysql://root:root1234@127.0.0.1:3306/forcedb")
//             .await
//             .expect("connect db error");
//         let mint_tasks = vec![
//             MintTask {
//                 block_number: 1,
//                 lock_tx_hash: "6f6d3b16f97a3910da97b863deb6408a3e636f8a5fe30f2e66ad1a8ec6b96d68"
//                     .to_owned(),
//                 lock_tx_proof: "proof".to_owned(),
//             },
//             MintTask {
//                 block_number: 2,
//                 lock_tx_hash: "9f6d3b16f97a3910da97b863deb6408a3e636f8a5fe30f2e66ad1a8ec6b96d68"
//                     .to_owned(),
//                 lock_tx_proof: "proof2".to_owned(),
//             },
//         ];
//         store_mint_tasks(&pool, &mint_tasks).await.unwrap();
//     }
//
//     #[tokio::test]
//     async fn test_get_retry_tasks() {
//         let pool = MySqlPool::connect("mysql://root:root1234@127.0.0.1:3306/forcedb")
//             .await
//             .expect("connect db error");
//         let tasks = get_retry_tasks(&pool).await.unwrap();
//         println!("retry tasks: {:?}", tasks);
//     }
//
//     #[tokio::test]
//     async fn test_update_relayed_tx() {
//         let pool = MySqlPool::connect("mysql://root:root1234@127.0.0.1:3306/forcedb")
//             .await
//             .expect("connect db error");
//         let is_succeed = update_relayed_tx(
//             &pool,
//             "6f6d3b16f97a3910da97b863deb6408a3e636f8a5fe30f2e66ad1a8ec6b96d68".to_string(),
//             "retryable".to_string(),
//             "network timeout".to_string(),
//         )
//         .await
//         .unwrap();
//         assert!(is_succeed, true);
//     }
//
//     #[tokio::test]
//     async fn test_delete_relayed_tx() {
//         let pool = MySqlPool::connect("mysql://root:root1234@127.0.0.1:3306/forcedb")
//             .await
//             .expect("connect db error");
//         let is_succeed = delete_relayed_tx(
//             &pool,
//             "9f6d3b16f97a3910da97b863deb6408a3e636f8a5fe30f2e66ad1a8ec6b96d68".to_string(),
//         )
//         .await
//         .unwrap();
//         assert!(is_succeed, true);
//     }
// }
