use super::types::*;
use crate::transfer::to_ckb::create_bridge_cell;
use crate::util::ckb_util::{parse_privkey, Generator};
use crate::util::eth_util::convert_eth_address;
use crate::util::settings::Settings;
use ckb_jsonrpc_types::TransactionView;
use ckb_sdk::Address;
use ckb_types::packed::Script;
use force_sdk::util::{ensure_indexer_sync, parse_privkey_path};
use jsonrpc_core::{IoHandler, Result};
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use secp256k1::SecretKey;
use std::str::FromStr;

#[rpc]
pub trait Rpc {
    fn create_bridge_cell(&self, args: CreateBridgeCellArgs) -> Result<CreateBridgeCellResponse>;

    fn burn(&self, args: BurnArgs) -> Result<TransactionView>;
}

pub struct RpcImpl {
    config_path: String,
    indexer_url: String,
    ckb_rpc_url: String,
    settings: Settings,
    private_key_path: String,
    from_privkey: SecretKey,
    from_lockscript: Script,
}

impl RpcImpl {
    fn new(
        config_path: String,
        indexer_url: String,
        ckb_rpc_url: String,
        private_key_path: String,
    ) -> Result<Self> {
        let settings = Settings::new(config_path.as_str()).expect("invalid settings");
        let from_privkey = parse_privkey_path(&private_key_path).expect("invalid private key path");
        let from_lockscript = parse_privkey(&from_privkey);
        Ok(Self {
            private_key_path,
            config_path,
            indexer_url,
            ckb_rpc_url,
            settings,
            from_privkey,
            from_lockscript,
        })
    }

    fn get_generator(&self) -> Result<Generator> {
        let mut generator = Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.settings.clone(),
        )
        .map_err(|e| {
            jsonrpc_core::Error::invalid_params(format!("new geneartor fail, err: {}", e))
        })?;
        ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60).map_err(
            |e| {
                jsonrpc_core::Error::invalid_params(format!(
                    "failed to ensure indexer sync : {}",
                    e
                ))
            },
        )?;
        Ok(generator)
    }
}

impl Rpc for RpcImpl {
    fn create_bridge_cell(&self, args: CreateBridgeCellArgs) -> Result<CreateBridgeCellResponse> {
        let tx_fee = "0.1".to_string();
        let capacity = "283".to_string();
        let outpoint = create_bridge_cell(
            self.config_path.clone(),
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            self.private_key_path.clone(),
            tx_fee,
            capacity,
            args.eth_token_address,
            args.recipient_address,
            args.bridge_fee,
        )
        .map_err(|e| {
            jsonrpc_core::Error::invalid_params(format!("fail to create bridge cell, err: {}", e))
        })?;
        Ok(CreateBridgeCellResponse { outpoint })
    }

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
    private_key_path: String,
    listen_url: String,
    threads_num: usize,
) {
    let mut io = IoHandler::new();
    let rpc = RpcImpl::new(config_path, indexer_url, ckb_rpc_url, private_key_path)
        .expect("init handler error");
    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io)
        .threads(threads_num)
        .start_http(&listen_url.parse().unwrap())
        .unwrap();
    server.wait();
}
