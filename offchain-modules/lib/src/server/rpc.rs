use super::types::*;
use crate::transfer::to_ckb::create_bridge_cell;
use crate::util::ckb_util::{
    build_lockscript_from_address, parse_cell, parse_main_chain_headers, Generator,
};
use crate::util::eth_util::{
    build_lock_eth_payload, build_lock_token_payload, convert_eth_address, make_transaction,
    rlp_transaction, Web3Client,
};
use crate::util::settings::Settings;
use ckb_jsonrpc_types::{Uint128, Uint64};
use ckb_sdk::{Address, HumanCapacity};
use ckb_types::packed::Script;
use ethabi::Token;
use force_sdk::cell_collector::get_live_cell_by_typescript;
use force_sdk::util::ensure_indexer_sync;
use jsonrpc_core::{IoHandler, Result};
use jsonrpc_derive::rpc;
use jsonrpc_http_server::ServerBuilder;
use molecule::prelude::Entity;
use std::str::FromStr;
use tokio::runtime::Runtime;
use web3::types::{H160, U256};

#[rpc]
pub trait Rpc {
    #[rpc(name = "create_bridge_cell")]
    fn create_bridge_cell(&self, args: CreateBridgeCellArgs) -> Result<CreateBridgeCellResponse>;
    #[rpc(name = "burn")]
    fn burn(&self, args: BurnArgs) -> Result<BurnResult>;
    #[rpc(name = "lock")]
    fn lock(&self, args: LockArgs) -> Result<LockResult>;
    #[rpc(name = "get_sudt_balance")]
    fn get_sudt_balance(&self, args: GetSudtBalanceArgs) -> Result<Uint128>;
    #[rpc(name = "get_best_block_height")]
    fn get_best_block_height(&self, args: GetBestBlockHeightArgs) -> Result<Uint64>;
}

pub struct RpcImpl {
    config_path: String,
    indexer_url: String,
    ckb_rpc_url: String,
    eth_rpc_url: String,
    settings: Settings,
    private_key_path: String,
}

impl RpcImpl {
    fn new(
        config_path: String,
        indexer_url: String,
        ckb_rpc_url: String,
        eth_rpc_url: String,
        private_key_path: String,
    ) -> Result<Self> {
        let settings = Settings::new(config_path.as_str()).expect("invalid settings");
        Ok(Self {
            private_key_path,
            config_path,
            indexer_url,
            ckb_rpc_url,
            eth_rpc_url,
            settings,
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
            args.bridge_fee.into(),
        )
        .map_err(|e| {
            jsonrpc_core::Error::invalid_params(format!("fail to create bridge cell, err: {}", e))
        })?;
        Ok(CreateBridgeCellResponse { outpoint })
    }

    fn burn(&self, args: BurnArgs) -> Result<BurnResult> {
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
        let lock_contract_address =
            convert_eth_address(self.settings.eth_token_locker_addr.as_str()).map_err(|_| {
                jsonrpc_core::Error::invalid_params("lock contract address parse fail")
            })?;
        let recipient_address = convert_eth_address(args.recipient_address.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("recipient address parse fail"))?;

        let mut generator = self.get_generator()?;

        let tx_fee: u64 = HumanCapacity::from_str(&args.tx_fee)
            .map_err(|_| jsonrpc_core::Error::invalid_params("tx fee parse fail"))?
            .into();

        let tx = generator
            .burn(
                tx_fee,
                from_lockscript,
                args.unlock_fee.into(),
                args.amount.into(),
                token_address,
                lock_contract_address,
                recipient_address,
            )
            .map_err(|e| jsonrpc_core::Error::invalid_params(format!("burn fail, err: {}", e)))?;
        let rpc_tx = ckb_jsonrpc_types::TransactionView::from(tx);
        Ok(BurnResult { tx: rpc_tx })
    }

    fn get_sudt_balance(&self, args: GetSudtBalanceArgs) -> Result<Uint128> {
        let token_address = convert_eth_address(args.token_address.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("token address parse fail"))?;
        let lock_contract_address =
            convert_eth_address(self.settings.eth_token_locker_addr.as_str()).map_err(|_| {
                jsonrpc_core::Error::invalid_params("lock contract address parse fail")
            })?;

        let mut generator = self.get_generator()?;

        let balance = generator
            .get_sudt_balance(args.address, token_address, lock_contract_address)
            .map_err(|e| {
                jsonrpc_core::Error::invalid_params(format!("get_sudt_balance fail, err: {}", e))
            })?;
        Ok(balance.into())
    }

    fn lock(&self, args: LockArgs) -> Result<LockResult> {
        let to = convert_eth_address(self.settings.eth_token_locker_addr.as_str())
            .map_err(|_| jsonrpc_core::Error::invalid_params("lock contract address parse fail"))?;
        let nonce = U256::from(u128::from(args.nonce));
        let gas_price = U256::from(u128::from(args.gas_price));
        let amount = U256::from(u128::from(args.amount));
        let bridge_fee = U256::from(u128::from(args.bridge_fee));

        let token_addr = convert_eth_address(&args.token_address)
            .map_err(|_| jsonrpc_core::Error::invalid_params("token address parse fail"))?;
        let recipient_lockscript = build_lockscript_from_address(&args.ckb_recipient_address)
            .map_err(|_| jsonrpc_core::Error::invalid_params("ckb recipient address parse fail"))?;

        let data = [
            Token::Address(token_addr),
            Token::Uint(amount),
            Token::Uint(bridge_fee),
            Token::Bytes(recipient_lockscript.as_slice().to_vec()),
            Token::Bytes(hex::decode(args.replay_resist_outpoint).map_err(|e| {
                jsonrpc_core::Error::invalid_params(format!(
                    "decode replay_resist_outpoint fail, err: {}",
                    e
                ))
            })?),
            Token::Bytes(hex::decode(args.sudt_extra_data).map_err(|e| {
                jsonrpc_core::Error::invalid_params(format!(
                    "decode sudt_extra_data fail, err: {}",
                    e
                ))
            })?),
        ];

        let mut eth_value = amount;

        let input_data = {
            if token_addr.0 == [0u8; 20] {
                let lock_eth_data = &data[2..];
                build_lock_eth_payload(lock_eth_data).map_err(|e| {
                    jsonrpc_core::Error::invalid_params(format!(
                        "abi encode lock eth data fail, err: {}",
                        e
                    ))
                })?
            } else {
                eth_value = U256::from(0);
                build_lock_token_payload(&data).map_err(|e| {
                    jsonrpc_core::Error::invalid_params(format!(
                        "abi encode lock token data fail, err: {}",
                        e
                    ))
                })?
            }
        };
        let raw_transaction = make_transaction(to, nonce, input_data, gas_price, eth_value);
        let result = LockResult {
            nonce: raw_transaction.nonce,
            to: raw_transaction.to,
            value: raw_transaction.value,
            gas_price: raw_transaction.gas_price,
            gas: raw_transaction.gas,
            data: hex::encode(raw_transaction.clone().data),
            raw: rlp_transaction(&raw_transaction),
        };
        Ok(result)
    }

    fn get_best_block_height(&self, args: GetBestBlockHeightArgs) -> Result<Uint64> {
        match args.chain.as_str() {
            "ckb" => {
                let contract_address = convert_eth_address(&self.settings.eth_ckb_chain_addr)
                    .map_err(|e| {
                        jsonrpc_core::Error::invalid_params(format!(
                            "abi encode lock eth data fail, err: {}",
                            e
                        ))
                    })?;

                let mut eth_client = Web3Client::new(self.eth_rpc_url.clone());

                async fn get_best_block_height(
                    client: &mut Web3Client,
                    contract_address: H160,
                ) -> Result<u64> {
                    client
                        .get_contract_height("latestBlockNumber", contract_address)
                        .await
                        .map_err(|e| {
                            jsonrpc_core::Error::invalid_params(format!(
                                "eth client call get_contract_height, err: {}",
                                e
                            ))
                        })
                }

                let result = Runtime::new()
                    .unwrap()
                    .block_on(get_best_block_height(&mut eth_client, contract_address))?;

                Ok(Uint64::from(result))
            }
            "eth" => {
                let mut generator = self.get_generator()?;

                let typescript =
                    parse_cell(self.settings.light_client_cell_script.cell_script.as_str())
                        .map_err(|e| {
                            jsonrpc_core::Error::invalid_params(format!(
                                "get typescript fail {:?}",
                                e
                            ))
                        })?;

                let cell = get_live_cell_by_typescript(&mut generator.indexer_client, typescript)
                    .map_err(|_| jsonrpc_core::Error::invalid_params("get live cell fail"))?
                    .ok_or_else(|| {
                        jsonrpc_core::Error::invalid_params("eth header cell not exist")
                    })?;

                let (un_confirmed_headers, _) = parse_main_chain_headers(
                    cell.output_data.as_bytes().to_vec(),
                )
                .map_err(|_| jsonrpc_core::Error::invalid_params("parse header data fail"))?;

                let best_header = un_confirmed_headers
                    .last()
                    .ok_or_else(|| jsonrpc_core::Error::invalid_params("header is none"))?;
                let best_block_number = best_header
                    .number
                    .ok_or_else(|| jsonrpc_core::Error::invalid_params("header number is none"))?;
                Ok(Uint64::from(best_block_number.as_u64()))
            }
            _ => Err(jsonrpc_core::Error::invalid_params(
                "unknown chain type, only support eth and ckb",
            )),
        }
    }
}

pub fn start(
    config_path: String,
    ckb_rpc_url: String,
    eth_rpc_url: String,
    indexer_url: String,
    private_key_path: String,
    listen_url: String,
    threads_num: usize,
) {
    let mut io = IoHandler::new();
    let rpc = RpcImpl::new(
        config_path,
        indexer_url,
        ckb_rpc_url,
        eth_rpc_url,
        private_key_path,
    )
    .expect("init handler error");
    io.extend_with(rpc.to_delegate());

    log::info!("server start at {}", &listen_url);
    let server = ServerBuilder::new(io)
        .threads(threads_num)
        .start_http(&listen_url.parse().unwrap())
        .unwrap();
    server.wait();
}
