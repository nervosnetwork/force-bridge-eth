mod ckb_sign_util;
mod eth_sign_util;

use crate::ckb_sign_util::{
    get_live_cell_with_cache, get_privkey_signer, to_multisig_congif, MultisigConf, TxHelper,
};
use crate::eth_sign_util::{get_msg_signature, get_secret_key};
use ckb_sdk::HttpRpcClient;
use ckb_types::bytes::Bytes;
use ckb_types::packed;
use ckb_types::packed::{CellOutput, OutPoint};
use ckb_types::prelude::Entity;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;
use std::collections::HashMap;

fn main() {
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

fn sign_ckb_tx(args: Params) -> Result<Value> {
    let args: Result<Map<String, Value>> = args.parse();
    if let Ok(params) = args {
        if !params.clone().contains_key("multi_config")
            || !params.clone().contains_key("unsigned_tx")
        {
            return Err(Error::invalid_params("the request params is invalid."));
        }
        let multi_conf_raw = params.get("multi_config").unwrap().as_str().unwrap();
        let multi_conf: MultisigConf = serde_json::from_str(&multi_conf_raw).unwrap();
        let multi_config = to_multisig_congif(&multi_conf).unwrap();
        let tx: packed::Transaction = packed::Transaction::new_unchecked(
            hex::decode(params.get("multi_config").unwrap().as_str().unwrap())
                .unwrap()
                .into(),
        );
        let tx_view = tx.into_view();
        let mut tx_helper = TxHelper::new(tx_view);
        tx_helper.add_multisig_config(multi_config);
        let mut rpc_client = HttpRpcClient::new(String::from(""));
        let mut live_cell_cache: HashMap<(OutPoint, bool), (CellOutput, Bytes)> =
            Default::default();
        let mut get_live_cell_fn = |out_point: OutPoint, with_data: bool| {
            get_live_cell_with_cache(&mut live_cell_cache, &mut rpc_client, out_point, with_data)
                .map(|(output, _)| output)
        };
        let privkey =
            get_secret_key("c4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a")
                .map_err(|_| Error::internal_error())?;
        let signer = get_privkey_signer(privkey);
        let mut result = Map::new();
        for (lock_args, signature) in tx_helper
            .sign_inputs(signer, &mut get_live_cell_fn, true)
            .unwrap()
        {
            result.insert(
                hex::encode(lock_args),
                Value::String(hex::encode(signature)),
            );
        }
        Ok(Value::Object(result))
    } else {
        Ok(Value::String("hi".into()))
    }
}

fn sign_eth_tx(args: Params) -> Result<Value> {
    let args: Result<Map<String, Value>> = args.parse();
    println!("sign_eth_tx args: {:?}", args);
    let raw_tx: &Value;
    if let Ok(params) = args {
        if params.clone().contains_key("raw_tx") {
            raw_tx = params.get("raw_tx").unwrap();
            let mut raw_msg = [0u8; 32];
            let msg = hex::decode(raw_tx.as_str().unwrap()).unwrap();
            if msg.len() != 32 {
                return Err(Error::invalid_params("raw_tx_hash is invalid."));
            }
            raw_msg.copy_from_slice(&msg.as_slice());
            let privkey =
                get_secret_key("c4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a")
                    .map_err(|_| Error::internal_error())?;
            let signature =
                get_msg_signature(&raw_msg, privkey).map_err(|_| Error::internal_error())?;
            return Ok(Value::String(hex::encode(signature)));
        }
    }
    Err(Error::invalid_params("invalid params"))
}
