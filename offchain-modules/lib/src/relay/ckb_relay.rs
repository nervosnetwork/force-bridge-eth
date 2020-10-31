use crate::transfer::to_eth::get_add_ckb_headers_func;
use crate::util::ckb_util::Generator;
use crate::util::eth_util::Web3Client;
use anyhow::Result;
use ethabi::Token;
use ethereum_types::U256;
use log::info;
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
    pub async fn start(&mut self) -> Result<()> {
        let mut ckb_relay = Generator::new(
            self.ckb_rpc_url.clone(),
            self.indexer_url.clone(),
            Default::default(),
        )
        .map_err(|e| anyhow::anyhow!("failed to crate generator: {}", e))?;
        let mut web3_client = Web3Client::new(self.eth_rpc_url.clone());

        let mut block_height = web3_client
            .get_light_client_current_height(self.contract_addr)
            .await?;
        info!("blcok header number : {:?}", block_height);

        loop {
            block_height += 1;
            let headers = ckb_relay.get_ckb_headers(vec![block_height])?;
            info!("headers : {:?} ", hex::encode(headers.as_slice()));

            let add_headers_func = get_add_ckb_headers_func();
            let add_headers_abi = add_headers_func.encode_input(&[Token::Bytes(headers)])?;
            let result = web3_client
                .send_transaction(
                    self.from,
                    self.contract_addr,
                    self.priv_key_path.clone(),
                    add_headers_abi,
                    U256::from(0),
                )
                .await?;
            info!("tx_hash : {:?} \n", hex::encode(result));
            // TODO : use send_raw_transaction_with_confirmation replace thread sleep
            std::thread::sleep(std::time::Duration::from_secs(25));
        }
    }
}
