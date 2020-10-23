use anyhow::Result;

use web3::types::{H160, H256};
use crate::util::eth_util::{Web3Client, function_encode};
use ethabi::{Function, Param, ParamType};

pub fn approve(from: H160, to: H160, url: String, chain_id: u32) -> H256{
    let mut rpc_client = Web3Client::new(url, chain_id);
    let function = Function {
        name: "approve".to_owned(),
        inputs: vec![Param {
            name: "_spender".to_owned(),
            kind: ParamType::Address,
        }, Param {
            name: "_value".to_owned(),
            kind: ParamType::Uint(256),
        }],
        outputs: vec![
            Param {
                name: "success".to_owned(),
                kind: ParamType::Bool,
            }
        ],
        constant: false,
    };
    let data = function_encode(function);
    rpc_client.send_transaction(from, to, data)
}

pub fn lock(from: H160, to: H160, url: String, chain_id: u32) -> H256 {
    let mut rpc_client = Web3Client::new(url, chain_id);
    let function = Function {
        name: "lock".to_owned(),
        inputs: vec![Param {
            name: "token".to_owned(),
            kind: ParamType::Address,
        }, Param {
            name: "amount".to_owned(),
            kind: ParamType::Uint(256),
        }, Param {
            name: "ckbAddress".to_owned(),
            kind: ParamType::String,
        }],
        outputs: vec![],
        constant: false,
    };
    let data = function_encode(function);
    rpc_client.send_transaction(from, to, data)
}

pub fn parse_eth_proof() -> Result<()> {
    todo!()
}

pub fn verify_eth_spv_proof() -> bool {
    todo!()
}