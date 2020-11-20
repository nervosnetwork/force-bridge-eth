use crate::util::ckb_util::Generator;
use crate::util::eth_util::convert_eth_address;
use crate::util::settings::Settings;
use ckb_sdk::Address;
use ckb_types::packed::Script;
use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::ServerBuilder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BurnArgs {
    unlock_fee: u128,
    amount: u128,
    token_address: String,
    lock_contract_address: String,
    recipient_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonrpcContractArgs {
    from_lockscript_addr: String,
    tx_fee: u64,
    sub_cmd: BurnArgs,
}

pub fn start(
    config_path: String,
    rpc_url: String,
    indexer_url: String,
    listen_url: String,
    threads_num: usize,
) {
    let mut io = jsonrpc_core::IoHandler::new();
    io.add_method("burn", move |params: Params| {
        dbg!(&params);
        let rpc_args: JsonrpcContractArgs = params.parse().unwrap();
        let from_lockscript = Script::from(
            Address::from_str(rpc_args.from_lockscript_addr.as_str())
                .unwrap()
                .payload(),
        );
        let token_address = convert_eth_address(rpc_args.sub_cmd.token_address.as_str()).unwrap();
        let lock_contract_address =
            convert_eth_address(rpc_args.sub_cmd.lock_contract_address.as_str()).unwrap();
        let recipient_address =
            convert_eth_address(rpc_args.sub_cmd.recipient_address.as_str()).unwrap();

        let settings = Settings::new(config_path.as_str()).unwrap();
        let mut generator = Generator::new(rpc_url.clone(), indexer_url.clone(), settings).unwrap();

        let tx = generator
            .burn(
                rpc_args.tx_fee,
                from_lockscript,
                rpc_args.sub_cmd.unlock_fee,
                rpc_args.sub_cmd.amount,
                token_address,
                lock_contract_address,
                recipient_address,
            )
            .unwrap();
        let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx);
        Ok(serde_json::to_value(rpc_tx).unwrap())
    });

    let server = ServerBuilder::new(io)
        .threads(threads_num)
        .start_http(&listen_url.parse().unwrap())
        .unwrap();
    server.wait();
}
