use crate::util::ckb_util::Generator;
use ckb_sdk::HttpRpcClient;
use std::collections::HashSet;
use web3::types::{Bytes, TransactionRequest};

pub struct CKBRelayer {
    pub ckb_rpc_url: String,
    pub indexer_url: String,
}

impl CKBRelayer {
    pub fn new(ckb_rpc_url: String, indexer_url: String) -> Self {
        CKBRelayer {
            ckb_rpc_url,
            indexer_url,
        }
    }
    pub fn start(&mut self) {
        let mut ckb_relay =
            Generator::new(self.ckb_rpc_url.clone(), self.indexer_url.clone()).unwrap();
        let headers = ckb_relay.get_ckb_headers(vec![3609, 3676]);
        print!("{:?}\n", headers);
    }
}
