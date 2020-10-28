use crate::transfer::to_eth::relay_ckb_headers;
use crate::util::ckb_util::Generator;
use web3::types::H160;

pub struct CKBRelayer {
    pub ckb_rpc_url: String,
    pub indexer_url: String,
    pub eth_rpc_url: String,
    pub from: H160,
    pub contract_addr: H160,
    pub priv_key_path: String,
}

impl CKBRelayer {
    pub fn new(
        ckb_rpc_url: String,
        indexer_url: String,
        eth_rpc_url: String,
        from: H160,
        contract_addr: H160,
        priv_key_path: String,
    ) -> Self {
        CKBRelayer {
            ckb_rpc_url,
            indexer_url,
            eth_rpc_url,
            from,
            contract_addr,
            priv_key_path,
        }
    }
    pub fn start(&mut self) {
        let mut ckb_relay =
            Generator::new(self.ckb_rpc_url.clone(), self.indexer_url.clone()).unwrap();
        let headers = ckb_relay.get_ckb_headers(vec![36]);
        print!("headers : {:?} \n", hex::encode(headers.as_slice()));

        let result = relay_ckb_headers(
            self.from,
            self.contract_addr,
            self.eth_rpc_url.clone(),
            self.priv_key_path.clone(),
            headers,
        );
        print!("{:?}\n", hex::encode(result));
    }
}
