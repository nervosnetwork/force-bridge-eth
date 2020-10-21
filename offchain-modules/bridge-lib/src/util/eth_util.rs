
use web3::futures::Future;
use web3::types::{TransactionRequest, Block, U256, H256, H160};
use rlp::RlpStream;
use futures::future::join_all;
use hex;
use web3::transports::Http;
use web3::Web3;

pub struct Web3Client {
    url: String,
    client: Web3<Http>,
}

impl Web3Client {
    pub fn new(url: String) -> Web3Client {
        let client = {
            let (eloop, transport) = web3::transports::Http::new(
                url.as_str(),
            )
                .unwrap();
            eloop.into_remote();
            web3::Web3::new(transport)
        };
        Web3Client { url, client }
    }

    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn client(&mut self) -> &mut Web3<Http> {
        &mut self.client
    }

    pub fn send_transaction(&mut self, tx: TransactionRequest) -> H256 {
        let tx_hash = self.client.eth().send_transaction(tx).wait().unwrap();
        tx_hash
    }

    pub fn  get_blocks(
        &mut self,
        start: usize,
        stop: usize,
    ) -> (Vec<Vec<u8>>, Vec<H256>) {
        let futures = (start..stop)
            .map(|i| self.client.eth().block((i as u64).into()))
            .collect::<Vec<_>>();
        let block_headers = join_all(futures).wait().unwrap();

        let mut blocks: Vec<Vec<u8>> = vec![];
        let mut hashes: Vec<H256> = vec![];
        for block_header in block_headers {
            let mut stream = RlpStream::new();
            rlp_append(&block_header.clone().unwrap(), &mut stream);
            blocks.push(stream.out());
            hashes.push(H256(block_header.clone().unwrap().hash.unwrap().0.into()));
        }
        for i in 0..blocks.len() {
            println!("header rlp: {:?}",  hex::encode(blocks[i].clone()));
        }

        (blocks, hashes)
    }
}


pub fn make_transaction(from: H160, to: H160) -> TransactionRequest {
    TransactionRequest {
        from,
        to: Some(to),
        gas: None,
        gas_price: None,
        value: Some(U256::from(10000)),
        data: None,
        nonce: None,
        condition: None
    }
}

fn rlp_append<TX>(header: &Block<TX>, stream: &mut RlpStream) {
    stream.begin_list(15);
    stream.append(&header.parent_hash);
    stream.append(&header.uncles_hash);
    stream.append(&header.author);
    stream.append(&header.state_root);
    stream.append(&header.transactions_root);
    stream.append(&header.receipts_root);
    stream.append(&header.logs_bloom);
    stream.append(&header.difficulty);
    stream.append(&header.number.unwrap());
    stream.append(&header.gas_limit);
    stream.append(&header.gas_used);
    stream.append(&header.timestamp);
    stream.append(&header.extra_data.0);
    stream.append(&header.mix_hash.unwrap());
    stream.append(&header.nonce.unwrap());
}


