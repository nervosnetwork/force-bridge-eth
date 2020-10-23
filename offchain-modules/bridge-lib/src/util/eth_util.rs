
use web3::types::{Block, U256, H256, H160, Bytes};
use rlp::RlpStream;
use futures::future::join_all;
use hex;
use web3::transports::Http;
use web3::Web3;
use ethereum_tx_sign::RawTransaction;
use ethabi::{Function, Token};
use futures::Future;
use anyhow::{anyhow, Result};

pub struct Web3Client {
    url: String,
    chain_id: u32,
    client: Web3<Http>,
}

impl Web3Client {
    pub fn new(url: String, chain_id: u32) -> Web3Client {
        let client = {
            let (eloop, transport) = web3::transports::Http::new(
                url.as_str(),
            )
                .unwrap();
            eloop.into_remote();
            web3::Web3::new(transport)
        };
        Web3Client { url, chain_id, client }
    }

    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn client(&mut self) -> &mut Web3<Http> {
        &mut self.client
    }
    pub fn chain_id(&mut self) -> &u32 {& self.chain_id}

    pub fn send_transaction(&mut self, from: H160, to: H160, key_path: String, data: Vec<u8>) -> H256 {
        let nonce = self.client().eth().transaction_count(from.clone(), None).wait().unwrap();
        let tx = make_transaction(to, nonce, data);
        let signed_tx = tx.sign(&parse_private_key(key_path.as_str()).unwrap(), &self.chain_id);
        let tx_hash = self.client().eth().send_raw_transaction(Bytes::from(signed_tx)).wait().unwrap();
        println!("tx hash: {:?}", tx_hash);
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

pub fn function_encode(function: Function) -> Vec<u8>{
    let func = Function::from(function);
    let mut uint = [0u8; 32];
    uint[31] = 69;
    func.encode_input(&[Token::Uint(uint.into()), Token::Bool(true)]).unwrap()
}


pub fn make_transaction(to: H160, nonce: U256, data: Vec<u8>) -> RawTransaction {
    use ethereum_types::U256;
    RawTransaction {
        nonce: convert_u256(nonce),
        to: Some(convert_account(to)),
        value: U256::from(10000),
        gas_price: U256::from(1000000000),
        gas: U256::from(21000),
        data
    }
}

pub fn parse_private_key(path: &str) -> Result<ethereum_types::H256> {
    let content = std::fs::read_to_string(path)?;
    let private_key_string = content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("File is empty"))?;
    return Ok(ethereum_types::H256::from_slice(hex::decode(private_key_string).expect("invalid private_key_string.").as_slice()));
}

fn convert_u256(value: web3::types::U256) -> ethereum_types::U256 {
    let web3::types::U256(ref arr) = value;
    let mut ret = [0; 4];
    ret[0] = arr[0];
    ret[1] = arr[1];
    ethereum_types::U256(ret)
}

fn convert_account(value: web3::types::H160) -> ethereum_types::H160 {
    ethereum_types::H160::from(value.0)
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


