use crate::util::eth_util::Web3Client;
use ethabi::{Function, Param, ParamType, Token};
use tokio::runtime::Runtime;
use web3::types::{H160, H256, U256};

pub fn approve(from: H160, to: H160, url: String, key_path: String) -> H256 {
    let mut rpc_client = Web3Client::new(url);
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
    let tokens = [Token::Address(from), Token::Uint(U256::max_value())];
    let input_data = function.encode_input(&tokens).expect("invalid input data");
    let f = rpc_client.send_transaction(from, to, key_path, input_data, U256::from(0));
    let mut rt = Runtime::new().unwrap();
    rt.block_on(f).expect("invalid tx hash")
}

pub fn lock_token(from: H160, to: H160, url: String, key_path: String, data: &[Token]) -> H256 {
    let mut rpc_client = Web3Client::new(url);
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
    let input_data = function.encode_input(data).expect("invalid input data");
    let f = rpc_client.send_transaction(from, to, key_path, input_data, U256::from(0));
    let mut rt = Runtime::new().unwrap();
    rt.block_on(f).expect("invalid tx hash")
}

pub fn lock_eth(
    from: H160,
    to: H160,
    url: String,
    key_path: String,
    data: &[Token],
    eth_value: U256,
) -> H256 {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "lockETH".to_owned(),
        inputs: vec![Param {
            name: "ckbAddress".to_owned(),
            kind: ParamType::String,
        }],
        outputs: vec![],
        constant: false,
    };
    let input_data = function.encode_input(data).expect("invalid input data");
    let f = rpc_client.send_transaction(from, to, key_path, input_data, eth_value);
    let mut rt = Runtime::new().unwrap();
    rt.block_on(f).expect("invalid tx hash")
}

pub fn get_header_rlp(url: String, hash: H256) -> String {
    let mut rpc_client = Web3Client::new(url);
    let f = rpc_client.get_header_rlp_with_hash(hash);
    let mut rt = Runtime::new().unwrap();
    rt.block_on(f)
}

pub fn verify_eth_spv_proof() -> bool {
    todo!()
}
