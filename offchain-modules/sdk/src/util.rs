use crate::indexer::{Cell, IndexerRpcClient, Order, Pagination, ScriptType, SearchKey};
use anyhow::{anyhow, Result};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types as rpc_types;
use ckb_jsonrpc_types::{Script as JsonScript, Uint32};
use ckb_sdk::{
    calc_max_mature_number,
    constants::{CELLBASE_MATURITY, MIN_SECP_CELL_CAPACITY, ONE_CKB},
    HttpRpcClient, SignerFn, SECP256K1,
};
use ckb_types::{
    bytes::Bytes,
    core::{EpochNumberWithFraction, TransactionView},
    h256,
    packed::{CellOutput, OutPoint, Script},
    prelude::*,
    H160, H256,
};
use secp256k1::SecretKey;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

pub fn parse_privkey_path(path: &str) -> Result<secp256k1::SecretKey> {
    let content = std::fs::read_to_string(path)?;
    let privkey_string = content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("File is empty"))?;
    let privkey_bytes = hex::decode(clear_0x(privkey_string))?;
    Ok(secp256k1::SecretKey::from_slice(&privkey_bytes)?)
}

// Get max mature block number
pub fn get_max_mature_number(rpc_client: &mut HttpRpcClient) -> Result<u64, String> {
    let tip_epoch = rpc_client
        .get_tip_header()
        .map(|header| EpochNumberWithFraction::from_full_value(header.inner.epoch.0))?;
    let tip_epoch_number = tip_epoch.number();
    if tip_epoch_number < 4 {
        // No cellbase live cell is mature
        Ok(0)
    } else {
        let max_mature_epoch = rpc_client
            .get_epoch_by_number(tip_epoch_number - 4)?
            .ok_or_else(|| "Can not get epoch less than current epoch number".to_string())?;
        let start_number = max_mature_epoch.start_number;
        let length = max_mature_epoch.length;
        Ok(calc_max_mature_number(
            tip_epoch,
            Some((start_number, length)),
            CELLBASE_MATURITY,
        ))
    }
}

pub fn is_mature(info: &Cell, max_mature_number: u64) -> bool {
    let tx_index: u32 = info.tx_index.into();
    let block_number: u64 = info.block_number.into();
    // Not cellbase cell
    tx_index > 0
        // Live cells in genesis are all mature
        || block_number == 0
        || block_number <= max_mature_number
}

pub fn get_live_cell(
    client: &mut HttpRpcClient,
    out_point: OutPoint,
    with_data: bool,
) -> Result<(CellOutput, Bytes), String> {
    let cell = client.get_live_cell(out_point.clone(), with_data)?;
    if cell.status != "live" {
        return Err(format!(
            "Invalid cell status: {}, out_point: {}",
            cell.status, out_point
        ));
    }
    let cell_status = cell.status.clone();
    cell.cell
        .map(|cell| {
            (
                cell.output.into(),
                cell.data
                    .map(|data| data.content.into_bytes())
                    .unwrap_or_default(),
            )
        })
        .ok_or_else(|| {
            format!(
                "Invalid input cell, status: {}, out_point: {}",
                cell_status, out_point
            )
        })
}

#[allow(clippy::mutable_key_type)]
pub fn get_live_cell_with_cache(
    cache: &mut HashMap<(OutPoint, bool), (CellOutput, Bytes)>,
    client: &mut HttpRpcClient,
    out_point: OutPoint,
    with_data: bool,
) -> Result<(CellOutput, Bytes), String> {
    if let Some(output) = cache.get(&(out_point.clone(), with_data)).cloned() {
        Ok(output)
    } else {
        let output = get_live_cell(client, out_point.clone(), with_data)?;
        cache.insert((out_point, with_data), output.clone());
        Ok(output)
    }
}

pub fn get_live_cells_by_lockscript(
    _indexer_client: &mut IndexerRpcClient,
    _need_capacity: u64,
    lockscript: Script,
) -> Result<(Vec<Cell>, u64)> {
    let rpc_lock: JsonScript = lockscript.into();
    let _search_key = SearchKey {
        script: rpc_lock,
        script_type: ScriptType::Lock,
        args_len: None,
    };
    todo!()
}

pub fn get_live_cells<F: FnMut(usize, &Cell) -> (bool, bool)>(
    indexer_client: &mut IndexerRpcClient,
    search_key: SearchKey,
    mut terminator: F,
) -> Result<Vec<Cell>, String> {
    let limit = Uint32::from(100u32);
    let mut infos = Vec::new();
    let mut _cursor = None;
    loop {
        let live_cells: Pagination<Cell> =
            indexer_client.get_cells(search_key.clone(), Order::Asc, limit, None)?;
        if live_cells.objects.is_empty() {
            break;
        }
        _cursor = Some(live_cells.last_cursor);
        for (index, cell) in live_cells.objects.into_iter().enumerate() {
            let (stop, push_info) = terminator(index, &cell);
            if push_info {
                infos.push(cell);
            }
            if stop {
                return Ok(infos);
            }
        }
    }

    Ok(infos)
}

pub fn get_privkey_signer(privkey: SecretKey) -> SignerFn {
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let lock_arg = H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20])
        .expect("Generate hash(H160) from pubkey failed");
    Box::new(
        move |lock_args: &HashSet<H160>, message: &H256, _tx: &rpc_types::Transaction| {
            if lock_args.contains(&lock_arg) {
                if message == &h256!("0x0") {
                    Ok(Some([0u8; 65]))
                } else {
                    let message = secp256k1::Message::from_slice(message.as_bytes())
                        .expect("Convert to secp256k1 message failed");
                    let signature = SECP256K1.sign_recoverable(&message, &privkey);
                    Ok(Some(serialize_signature(&signature)))
                }
            } else {
                Ok(None)
            }
        },
    )
}

pub fn serialize_signature(signature: &secp256k1::recovery::RecoverableSignature) -> [u8; 65] {
    let (recov_id, data) = signature.serialize_compact();
    let mut signature_bytes = [0u8; 65];
    signature_bytes[0..64].copy_from_slice(&data[0..64]);
    signature_bytes[64] = recov_id.to_i32() as u8;
    signature_bytes
}

pub fn check_capacity(capacity: u64, to_data_len: usize) -> Result<(), String> {
    if capacity < MIN_SECP_CELL_CAPACITY {
        return Err(format!(
            "Capacity can not less than {} shannons",
            MIN_SECP_CELL_CAPACITY
        ));
    }
    if capacity < MIN_SECP_CELL_CAPACITY + (to_data_len as u64 * ONE_CKB) {
        return Err(format!(
            "Capacity can not hold {} bytes of data",
            to_data_len
        ));
    }
    Ok(())
}

pub async fn send_tx_sync(
    rpc_client: &mut HttpRpcClient,
    tx: &TransactionView,
    timeout: u64,
) -> Result<H256, String> {
    let tx_hash = rpc_client
        .send_transaction(tx.data())
        .map_err(|err| format!("Send transaction error: {}", err))?;
    assert_eq!(tx.hash(), tx_hash.pack());
    for i in 0..timeout {
        let status = rpc_client
            .get_transaction(tx_hash.clone())?
            .map(|t| t.tx_status.status);
        log::info!(
            "waiting for tx {} to be committed, loop index: {}, status: {:?}",
            &tx_hash,
            i,
            status
        );
        if status == Some(ckb_jsonrpc_types::Status::Committed) {
            return Ok(tx_hash);
        }
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
    Err(format!("tx {} not commited", &tx_hash))
}

pub async fn ensure_indexer_sync(
    rpc_client: &mut HttpRpcClient,
    indexer_client: &mut IndexerRpcClient,
    timeout: u64,
) -> Result<(), String> {
    let rpc_tip = rpc_client.get_tip_block_number()?;
    for _ in 0..timeout {
        let indexer_tip = indexer_client
            .get_tip()?
            .map(|t| t.block_number.value())
            .unwrap_or(0);
        log::info!("rpc_tip: {}, indexer_tip: {}", rpc_tip, indexer_tip);
        if indexer_tip >= rpc_tip {
            return Ok(());
        }
        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}
