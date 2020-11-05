use anyhow::{anyhow, Result};
use ethabi::{FixedBytes, Uint};
use ethereum_tx_sign::RawTransaction;
use log::{debug, info};
use rlp::RlpStream;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::types::{Address, Block, BlockId, Bytes, H160, H256, U256};
use web3::Web3;

pub const ETH_ADDRESS_LENGTH: usize = 40;

pub struct Web3Client {
    url: String,
    client: Web3<Http>,
}

impl Web3Client {
    pub fn new(url: String) -> Web3Client {
        let client = {
            let transport = web3::transports::Http::new(url.as_str()).expect("new transport");
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

    pub async fn send_transaction(
        &mut self,
        from: H160,
        to: H160,
        key_path: String,
        data: Vec<u8>,
        eth_value: U256,
    ) -> Result<H256> {
        let nonce = self.client().eth().transaction_count(from, None).await?;
        info!("tx current nonce :{}", &nonce);
        let chain_id = self.client().eth().chain_id().await?;
        debug!("chain id :{}", &chain_id);
        let tx = make_transaction(to, nonce, data, eth_value);
        let signed_tx = tx.sign(&parse_private_key(&key_path)?, &chain_id.as_u32());
        let tx_hash = self
            .client()
            .eth()
            // .send_raw_transaction_with_confirmation(Bytes::from(signed_tx), Duration::new(5, 100), 10)
            .send_raw_transaction(Bytes::from(signed_tx))
            .await?;
        debug!("tx hash: {:?}", tx_hash);
        Ok(tx_hash)
    }

    pub async fn get_block(&mut self, hash_or_number: BlockId) -> Result<Block<H256>> {
        let block = self.client.eth().block(hash_or_number).await?;
        match block {
            Some(block) => Ok(block),
            None => anyhow::bail!("the block is not exist."),
        }
    }

    pub async fn get_header_rlp(&mut self, hash_or_number: BlockId) -> Result<String> {
        let block = self.get_block(hash_or_number).await;
        let mut stream = RlpStream::new();
        rlp_append(&block.unwrap(), &mut stream);
        Ok(hex::encode(stream.out().as_slice()))
    }

    pub async fn get_light_client_current_height(&mut self, contract_addr: Address) -> Result<u64> {
        let contract = Contract::from_json(
            self.client.eth(),
            contract_addr,
            include_bytes!("ckb_chain_abi.json"),
        )
        .map_err(|e| anyhow::anyhow!("failed to instantiate contract by parse abi: {}", e))?;
        let result = contract.query("latestBlockNumber", (), None, Options::default(), None);
        let latest_block_number: u64 = result.await?;
        Ok(latest_block_number)
    }
    pub async fn get_mock_data(&mut self, contract_addr: Address) -> Result<Bytes> {
        let contract = Contract::from_json(
            self.client.eth(),
            contract_addr,
            include_bytes!("ckb_chain_abi.json"),
        )
        .map_err(|e| anyhow::anyhow!("failed to instantiate contract by parse abi: {}", e))?;
        let result = contract.query("rawHeaders", (), None, Options::default(), None);
        let mock_data: Bytes = result.await?;
        Ok(mock_data)
    }

    pub async fn is_header_exist(
        &mut self,
        block_number: u64,
        latest_header_hash: ckb_types::H256,
        contract_addr: Address,
    ) -> Result<bool> {
        let contract = Contract::from_json(
            self.client.eth(),
            contract_addr,
            include_bytes!("ckb_chain_abi.json"),
        )
        .map_err(|e| anyhow::anyhow!("failed to instantiate contract by parse abi: {}", e))?;

        info!(
            "ckb block {:?} header hash : {:?}",
            block_number,
            hex::encode(latest_header_hash.as_bytes())
        );

        // TODO : confirm the result contains all the block hash at same height(maybe fork)
        let result = contract.query(
            "getHeaderHash",
            Uint::from(block_number),
            None,
            Options::default(),
            None,
        );

        let header_hash: FixedBytes = result.await?;
        info!(
            "contact block {:?} header hash : {:?}",
            block_number,
            hex::encode(header_hash.as_slice())
        );
        if header_hash.as_slice() == latest_header_hash.as_bytes() {
            return Ok(true);
        }
        Ok(false)
    }
}

pub fn make_transaction(to: H160, nonce: U256, data: Vec<u8>, eth_value: U256) -> RawTransaction {
    RawTransaction {
        nonce: convert_u256(nonce),
        to: Some(convert_account(to)),
        value: eth_value,
        gas_price: U256::from(1000000000),
        gas: U256::from(2100000),
        data,
    }
}

pub fn parse_private_key(path: &str) -> Result<ethereum_types::H256> {
    let content = std::fs::read_to_string(path)?;
    let private_key_string = content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("File is empty"))?;
    return Ok(ethereum_types::H256::from_slice(
        hex::decode(private_key_string)?.as_slice(),
    ));
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
    stream.append(&header.logs_bloom.unwrap());
    stream.append(&header.difficulty);
    stream.append(&header.number.unwrap());
    stream.append(&header.gas_limit);
    stream.append(&header.gas_used);
    stream.append(&header.timestamp);
    stream.append(&header.extra_data.0);
    stream.append(&header.mix_hash.unwrap());
    stream.append(&header.nonce.unwrap());
}

pub fn convert_eth_address(mut address: &str) -> Result<H160> {
    if address.starts_with("0x") || address.starts_with("0X") {
        address = &address[2..];
    }
    if address.len() != ETH_ADDRESS_LENGTH {
        anyhow::bail!("invalid eth address: {:?}", address)
    }
    Ok(H160::from_slice(hex::decode(address)?.as_slice()))
}

#[tokio::test]
async fn test_get_block() {
    use cmd_lib::run_cmd;
    use web3::types::BlockNumber;
    let mut client = Web3Client::new(String::from(
        "https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2",
    ));
    let res = client.get_header_rlp((U64::from(3)).into()).await;
    println!("{:?}", res);
    let header_rlp = format!("0x{}", res.unwrap());
    println!("{:?}", header_rlp);
    run_cmd!(src/vendor/relayer ${header_rlp} > /tmp/data.json);
}
