pub struct ETHRelayer {
    pub ckb_rpc_url: String,
    pub eth_rpc_url: String,
    pub indexer_url: String,
    pub priv_key_path: String,
}

impl ETHRelayer {
    pub fn new(
        ckb_rpc_url: String,
        eth_rpc_url: String,
        indexer_url: String,
        priv_key_path: String,
    ) -> Self {
        ETHRelayer {
            ckb_rpc_url,
            eth_rpc_url,
            indexer_url,
            priv_key_path,
        }
    }

    pub fn start() {}
}
