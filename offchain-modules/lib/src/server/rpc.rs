use crate::util::ckb_util::Generator;
use crate::util::eth_util::convert_eth_address;
use crate::util::settings::Settings;
use ckb_jsonrpc_types::TransactionView;
use ckb_sdk::Address;
use ckb_types::packed::Script;
use jsonrpc_core::{IoHandler, Result};
use jsonrpc_derive::rpc;

use jsonrpc_http_server::ServerBuilder;
use std::str::FromStr;

use super::types::BurnArgs;

#[rpc]
pub trait Rpc {
    /// Adds two numbers and returns a result
    #[rpc(name = "burn")]
    fn burn(&self, args: BurnArgs) -> Result<TransactionView>;
}

pub struct RpcImpl {
    config_path: String,
    indexer_url: String,
    ckb_rpc_url: String,
}

impl Rpc for RpcImpl {
    fn burn(&self, args: BurnArgs) -> Result<TransactionView> {
        let from_lockscript = Script::from(
            Address::from_str(args.from_lockscript_addr.as_str())
                .map_err(|err| {
                    jsonrpc_core::Error::invalid_params_with_details(
                        err,
                        "ckb_address to script fail",
                    )
                })?
                .payload(),
        );
        let token_address = convert_eth_address(args.token_address.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("token address parse fail"))?;
        let lock_contract_address = convert_eth_address(args.lock_contract_address.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("lock contract address parse fail"))?;
        let recipient_address = convert_eth_address(args.recipient_address.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("recipient address parse fail"))?;

        let settings = Settings::new(self.config_path.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("new setting fail"))?;
        let mut generator =
            Generator::new(self.ckb_rpc_url.clone(), self.indexer_url.clone(), settings)
                .map_err(|_| jsonrpc_core::Error::invalid_params("new geneartor fail"))?;

        let tx = generator
            .burn(
                args.tx_fee,
                from_lockscript,
                args.unlock_fee,
                args.amount,
                token_address,
                lock_contract_address,
                recipient_address,
            )
            .map_err(|_| jsonrpc_core::Error::invalid_params("burn fail"))?;
        Ok(tx.into())
    }
}

pub fn start(
    config_path: String,
    ckb_rpc_url: String,
    indexer_url: String,
    listen_url: String,
    threads_num: usize,
) {
    let mut io = IoHandler::new();
    let rpc = RpcImpl {
        config_path,
        indexer_url,
        ckb_rpc_url,
    };
    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(threads_num)
        .start_http(&listen_url.parse().unwrap())
        .unwrap();
    server.wait();
}
