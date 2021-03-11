mod ckb_sign_util;
mod config;
mod eth_sign_util;
mod rocksdb_store;
mod types;

use crate::ckb_sign_util::{
    generate_from_lockscript, get_live_cell_with_cache, get_privkey_signer, parse_cell,
    parse_merkle_cell_data, to_multisig_congif, MultisigConf, TxHelper,
};
use crate::config::SignServerConfig;
use crate::eth_sign_util::{get_msg_signature, get_secret_key, Web3Client};
// use crate::rocksdb_store::{RocksDBStore, RocksDBValue};
use crate::types::{IndexerArgs, Opts, ServerArgs, SubCommand};
use anyhow::{anyhow, Result};
use ckb_sdk::HttpRpcClient;
use ckb_types::bytes::Bytes;
use ckb_types::packed::{CellOutput, OutPoint};
use ckb_types::prelude::Entity;
use ckb_types::{packed, H256};
use clap::Clap;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::indexer::IndexerRpcClient;
use jsonrpc_http_server::jsonrpc_core::{Error, IoHandler, Params, Value};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, ServerBuilder};
use rocksdb::ops::Put;
use shellexpand::tilde;
use std::collections::HashMap;
use web3::types::U64;

pub const CONFIG_PATH: &str = "~/.sign_server/config.toml";

fn main() -> Result<()> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    dbg!(&opts);
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .core_threads(100)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async { handler(opts).await })
}

pub async fn handler(opt: Opts) -> Result<()> {
    match opt.subcmd {
        SubCommand::Server(args) => server_handle(args).await,
        SubCommand::Indexer(args) => indexer_handle(args).await,
    }
}

pub async fn server_handle(args: ServerArgs) -> Result<()> {
    let config = SignServerConfig {
        db_path: args.db_path,
        ckb_private_key_path: args.ckb_private_key_path,
        eth_private_key_path: args.eth_private_key_path,
    };
    let config_path = tilde(CONFIG_PATH).into_owned();
    config.write(config_path.as_str())?;
    let mut io = IoHandler::default();
    io.add_method("sign_ckb_tx", sign_ckb_tx);
    io.add_method("sign_eth_tx", sign_eth_tx);

    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
        ]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server");

    server.wait();
    Ok(())
}

pub async fn indexer_handle(args: IndexerArgs) -> Result<()> {
    let config_path = tilde(CONFIG_PATH).into_owned();
    let config =
        SignServerConfig::new(config_path.as_str()).map_err(|_| Error::internal_error())?;
    let mut eth_client = Web3Client::new(args.eth_rpc_url);
    let mut indexer_client = IndexerRpcClient::new(args.ckb_indexer_url);
    let cell_script = parse_cell(args.cell_script.as_str())?;
    let cell = get_live_cell_by_typescript(&mut indexer_client, cell_script.clone())
        .map_err(|err| anyhow::anyhow!(err))?
        .ok_or_else(|| anyhow::anyhow!("no cell found"))?;

    let last_cell_output_data = cell.output_data.as_bytes();

    let mut last_cell_latest_height = 0u64;

    let (start_height, mut smt_tree, rocksdb) = match last_cell_output_data.len() {
        0 => {
            let rocksdb_store = rocksdb_store::RocksDBStore::new(config.db_path.clone());
            (
                100 as u64,
                rocksdb_store::SMT::new(sparse_merkle_tree::H256::zero(), rocksdb_store.clone()),
                rocksdb_store
                    .db
                    .ok_or_else(|| anyhow!("db is not exist."))?,
            )
        }
        _ => {
            let (start_height, latest_height, merkle_root) =
                parse_merkle_cell_data(last_cell_output_data.to_vec())?;
            last_cell_latest_height = latest_height;
            let rocksdb_store = rocksdb_store::RocksDBStore::open(config.db_path.clone());
            (
                start_height,
                rocksdb_store::SMT::new(merkle_root.into(), rocksdb_store.clone()),
                rocksdb_store
                    .db
                    .ok_or_else(|| anyhow!("db is not exist."))?,
            )
        }
    };
    loop {
        let block_number = U64([start_height]);
        let mut key = [0u8; 32];
        let mut height = [0u8; 8];
        height.copy_from_slice(start_height.to_le_bytes().as_ref());
        key[..8].clone_from_slice(&height);

        let chain_block = eth_client.get_block(block_number.into()).await?;
        let chain_block_hash = chain_block
            .hash
            .ok_or_else(|| anyhow!("the block number is not exist."))?;
        smt_tree
            .update(key.into(), chain_block_hash.0.into())
            .map_err(|err| anyhow::anyhow!(err))?;
        let rocksdb_store = smt_tree.store_mut();
        rocksdb_store.commit();
        rocksdb
            .put(key, smt_tree.root().as_slice())
            .map_err(|err| anyhow!(err))?;
    }
}

#[allow(clippy::mutable_key_type)]
fn sign_ckb_tx(args: Params) -> jsonrpc_http_server::jsonrpc_core::Result<Value> {
    let config_path = tilde(CONFIG_PATH).into_owned();
    let config =
        SignServerConfig::new(config_path.as_str()).map_err(|_| Error::internal_error())?;
    use jsonrpc_http_server::jsonrpc_core::Result;
    log::info!("sign_ckb_tx request params: {:?}", args);
    let args: Result<Vec<String>> = args.parse();
    if let Ok(params) = args {
        if params.len() != 3 {
            return Err(Error::invalid_params("the request params is invalid."));
        }
        let multi_conf_raw = params[0].clone();
        let multi_conf: MultisigConf = serde_json::from_str(&multi_conf_raw)
            .map_err(|_| Error::invalid_params("invalid multi_conf."))?;
        let multi_config = to_multisig_congif(&multi_conf).map_err(|_| Error::internal_error())?;
        log::info!("multi_config: {:?}", multi_conf);
        let tx: packed::Transaction = packed::Transaction::new_unchecked(
            hex::decode(params[1].clone())
                .map_err(|_| Error::internal_error())?
                .into(),
        );
        let mut rpc_client = HttpRpcClient::new(params[2].clone());
        let tx_view = tx.into_view();
        // log::info!(
        //     "tx: \n{}",
        //     serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(
        //         tx_view.clone()
        //     ))
        //     .unwrap()
        // );
        let privkey = get_secret_key(config.ckb_private_key_path.as_str())
            .map_err(|_| Error::internal_error())?;
        for item in tx_view.inputs() {
            let op = item.previous_output();
            let hash = H256::from_slice(op.tx_hash().raw_data().to_vec().as_slice())
                .map_err(|_| Error::internal_error())?;
            let tx = rpc_client
                .get_transaction(hash)
                .map_err(|_| Error::internal_error())?
                .ok_or_else(|| Error::internal_error())?
                .transaction;
            let mut index = [0u8; 4];
            index.copy_from_slice(op.index().raw_data().to_vec().as_slice());
            let cell_output = &tx.inner.outputs[u32::from_le_bytes(index) as usize];
            let lockscript = cell_output.lock.clone();
            let script: packed::Script = packed::Script::from(lockscript);
            let from_scirpt =
                generate_from_lockscript(privkey).map_err(|_| Error::internal_error())?;
            if script.as_slice() == from_scirpt.as_slice() {
                // current transaction has the cell of the signer and refuses to sign
                log::warn!("the current transaction is at risk of being attacked");
                return Err(Error::invalid_params(
                    "invalid params. the current transaction is at risk of being attacked.",
                ));
            }
        }
        // verify the original signature
        let cell_output_data = tx_view.outputs_data().get_unchecked(0).raw_data();
        let (start_height, latest_height, merkle_root) =
            parse_merkle_cell_data(cell_output_data.to_vec())
                .map_err(|_| Error::internal_error())?;
        log::info!(
            "start_height: {:?}, latest_height: {:?}, merkle_root: {:?}",
            start_height,
            latest_height,
            merkle_root
        );
        let mut tx_helper = TxHelper::new(tx_view);
        tx_helper.add_multisig_config(multi_config);

        let mut live_cell_cache: HashMap<(OutPoint, bool), (CellOutput, Bytes)> =
            Default::default();
        let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
            get_live_cell_with_cache(&mut live_cell_cache, &mut rpc_client, out_point, with_data)
                .map(|(output, _)| output)
        };

        let signer = get_privkey_signer(privkey);
        let mut result = vec![];
        for (lock_args, signature) in tx_helper
            .sign_inputs(signer, &mut get_live_cell_fn, true)
            .map_err(|_| Error::internal_error())?
        {
            result.push(hex::encode(lock_args).into());
            result.push(hex::encode(signature).into());
        }
        log::info!("sign_ckb_tx result: {:?}", result);
        Ok(Value::Array(result))
    } else {
        Err(Error::invalid_params(
            "invalid params. expect string array.",
        ))
    }
}

fn sign_eth_tx(args: Params) -> jsonrpc_http_server::jsonrpc_core::Result<Value> {
    let config_path = tilde(CONFIG_PATH).into_owned();
    let config =
        SignServerConfig::new(config_path.as_str()).map_err(|_| Error::internal_error())?;
    use jsonrpc_http_server::jsonrpc_core::Result;
    log::info!("sign_eth_tx request params: {:?}", args);
    let args: Result<Vec<String>> = args.parse();
    if let Ok(params) = args {
        if params.len() == 1 {
            let mut raw_msg = [0u8; 32];
            let msg = hex::decode(params[0].clone())
                .map_err(|_| Error::invalid_params("raw_tx_hash is invalid"))?;
            if msg.len() != 32 {
                return Err(Error::invalid_params("raw_tx_hash is invalid."));
            }
            raw_msg.copy_from_slice(&msg.as_slice());
            let privkey = get_secret_key(config.eth_private_key_path.as_str())
                .map_err(|_| Error::internal_error())?;
            let signature =
                get_msg_signature(&raw_msg, privkey).map_err(|_| Error::internal_error())?;
            log::info!("signature: {:?}", hex::encode(signature.clone()));
            return Ok(Value::String(hex::encode(signature)));
        }
    }
    Err(Error::invalid_params(
        "invalid params. expect string array.",
    ))
}
