use super::types::BurnArgs;
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

#[rpc]
pub trait Rpc {
    fn burn(&self, args: BurnArgs) -> Result<TransactionView>;
}

pub struct RpcImpl {
    indexer_url: String,
    ckb_rpc_url: String,
    settings: Settings,
}

impl RpcImpl {
    fn new(config_path: String, indexer_url: String, ckb_rpc_url: String) -> Result<Self> {
        let settings = Settings::new(config_path.as_str()).expect("invalid settings");
        Ok(Self {
            indexer_url,
            ckb_rpc_url,
            settings,
        })
    }

    fn get_generator(&self) -> Result<Generator> {
        Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.settings.clone(),
        )
        .map_err(|e| jsonrpc_core::Error::invalid_params(format!("new geneartor fail, err: {}", e)))
    }
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

        let mut generator = self.get_generator()?;

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
            .map_err(|e| jsonrpc_core::Error::invalid_params(format!("burn fail, err: {}", e)))?;
        let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx);
        Ok(rpc_tx)
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
    let rpc = RpcImpl::new(config_path, indexer_url, ckb_rpc_url).expect("init handler error");
    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(threads_num)
        .start_http(&listen_url.parse().unwrap())
        .unwrap();
    server.wait();
}
