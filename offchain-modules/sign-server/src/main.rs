pub mod eth_sign_util;

use crate::eth_sign_util::{get_msg_signature, get_secret_key};
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::*;

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
    let v1: Result<Vec<String>> = args.parse();
    if v1.is_ok() {
        let v = v1.unwrap();
        Ok(Value::from(v[0].clone()))
    } else {
        Ok(Value::String("hi".into()))
    }
}

fn sign_eth_tx(args: Params) -> Result<Value> {
    let args: Result<Map<String, String>> = args.parse();
    if let Ok(params) = args {
        // let msg = params.
    }
    let mut raw_msg = [0u8; 16];
    raw_msg.copy_from_slice(&"output_data".as_bytes()[..16]);
    let privkey = get_secret_key("").map_err(|_| Error::internal_error())?;
    let mut signature =
        get_msg_signature(&raw_msg, privkey).map_err(|_| Error::internal_error())?;
    Ok(Value::String(hex::encode(signature)))
}
