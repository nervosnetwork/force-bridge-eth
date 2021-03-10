mod ckb_sign_util;
mod eth_sign_util;

use crate::ckb_sign_util::{
    generate_from_lockscript, get_live_cell_with_cache, get_privkey_signer, parse_merkle_cell_data,
    to_multisig_congif, MultisigConf, TxHelper,
};
use crate::eth_sign_util::{get_msg_signature, get_secret_key};
use ckb_sdk::HttpRpcClient;
use ckb_types::bytes::Bytes;
use ckb_types::packed::{CellOutput, OutPoint};
use ckb_types::prelude::Entity;
use ckb_types::{packed, H256};
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
use std::collections::HashMap;

fn main() {
    env_logger::init();
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
}

#[allow(clippy::mutable_key_type)]
fn sign_ckb_tx(args: Params) -> Result<Value> {
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
        let privkey =
            get_secret_key("/tmp/.sign_server/ckb_key").map_err(|_| Error::internal_error())?;
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

fn sign_eth_tx(args: Params) -> Result<Value> {
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
            let privkey =
                get_secret_key("/tmp/.sign_server/eth_key").map_err(|_| Error::internal_error())?;
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
