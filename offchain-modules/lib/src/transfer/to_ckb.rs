use anyhow::Result;

use crate::util::eth_util::{function_encode, Web3Client};
use ethabi::{Function, Param, ParamType};
use web3::types::{H160, H256};

pub fn approve(from: H160, to: H160, url: String, chain_id: u32, key_path: String) -> H256 {
    let mut rpc_client = Web3Client::new(url, chain_id);
    let function = Function {
        name: "approve".to_owned(),
        inputs: vec![
            Param {
                name: "_spender".to_owned(),
                kind: ParamType::Address,
            },
            Param {
                name: "_value".to_owned(),
                kind: ParamType::Uint(256),
            },
        ],
        outputs: vec![Param {
            name: "success".to_owned(),
            kind: ParamType::Bool,
        }],
        constant: false,
    };
    let data = function_encode(function);
    rpc_client.send_transaction(from, to, key_path, data)
}

pub fn lock(from: H160, to: H160, url: String, chain_id: u32, key_path: String) -> H256 {
    let mut rpc_client = Web3Client::new(url, chain_id);
    let function = Function {
        name: "lock".to_owned(),
        inputs: vec![
            Param {
                name: "token".to_owned(),
                kind: ParamType::Address,
            },
            Param {
                name: "amount".to_owned(),
                kind: ParamType::Uint(256),
            },
            Param {
                name: "ckbAddress".to_owned(),
                kind: ParamType::String,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let data = function_encode(function);
    rpc_client.send_transaction(from, to, key_path, data)
}

pub fn parse_eth_proof() -> Result<()> {
    todo!()
}

pub fn verify_eth_spv_proof() -> bool {
    todo!()
}
