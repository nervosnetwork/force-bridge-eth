use anyhow::Result;

use web3::types::{H160, H256};
use crate::util::eth_util::{ make_transaction, Web3Client};

pub fn approve(from: H160, to: H160, url: String) -> H256{
    let mut rpc_client = Web3Client::new(url);
    let approve_tx = make_transaction(from, to);
    rpc_client.send_transaction(approve_tx)
}

pub fn lock(from: H160, to: H160, url: String) -> H256 {
    let mut rpc_client = Web3Client::new(url);
    let lock_tx = make_transaction(from, to);
    rpc_client.send_transaction(lock_tx)
}

pub fn parse_eth_proof() -> Result<()> {
    todo!()
}

pub fn verify_eth_spv_proof() -> bool {
    todo!()
}