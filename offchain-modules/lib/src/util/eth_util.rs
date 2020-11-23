use anyhow::{anyhow, Result};
use eth_spv_lib::eth_types::my_keccak256;
use ethabi::{FixedBytes, Function, Param, ParamType, Token, Uint};
use ethereum_tx_sign::RawTransaction;
use log::{debug, info};
use rlp::{DecoderError, Rlp, RlpStream};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::time::Duration;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::types::{Address, Block, BlockHeader, BlockId, Bytes, H160, H256, U256};
use web3::Web3;

pub const ETH_ADDRESS_LENGTH: usize = 40;
const MAX_GAS_LIMIT: u64 = 3000000;

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
        to: H160,
        key_path: String,
        data: Vec<u8>,
        gas_price: U256,
        eth_value: U256,
        wait: bool,
    ) -> Result<H256> {
        let signed_tx = self
            .build_sign_tx(to, key_path, data, gas_price, eth_value)
            .await?;
        let tx_hash = if wait {
            let receipt = self
                .client()
                .send_raw_transaction_with_confirmation(
                    Bytes::from(signed_tx),
                    Duration::new(1, 0),
                    1,
                )
                .await?;
            debug!("receipt: {:?}", &receipt);
            let tx_hash = receipt.transaction_hash;
            let tx_status = receipt.status.expect("should return status");
            if tx_status.as_usize() == 0 {
                panic!("tx failed")
            };
            tx_hash
        } else {
            self.client()
                .eth()
                .send_raw_transaction(Bytes::from(signed_tx))
                .await?
        };
        debug!("tx hash: {:?}", tx_hash);
        Ok(tx_hash)
    }

    pub async fn build_sign_tx(
        &mut self,
        to: H160,
        key_path: String,
        data: Vec<u8>,
        gas_price: U256,
        eth_value: U256,
    ) -> Result<Vec<u8>> {
        let private_key = &parse_private_key(&key_path)?;
        let key = SecretKey::from_slice(&private_key.0).unwrap();
        let from = secret_key_address(&key);
        let nonce = self.client().eth().transaction_count(from, None).await?;
        info!("tx current nonce :{}", &nonce);
        let chain_id = self.client().eth().chain_id().await?;
        debug!("chain id :{}", &chain_id);
        let tx_gas_price;
        if gas_price.is_zero() {
            tx_gas_price = self.client.eth().gas_price().await?;
        } else {
            tx_gas_price = gas_price;
        }
        let tx = make_transaction(to, nonce, data, tx_gas_price, eth_value);
        let signed_tx = tx.sign(&parse_private_key(&key_path)?, &chain_id.as_u32());
        Ok(signed_tx)
    }

    pub async fn get_block(&mut self, hash_or_number: BlockId) -> Result<Block<H256>> {
        let block = self.client.eth().block(hash_or_number).await?;
        match block {
            Some(block) => Ok(block),
            None => anyhow::bail!("the block is not exist."),
        }
    }

    pub async fn get_header_rlp(&mut self, hash_or_number: BlockId) -> Result<String> {
        let block = self.get_block(hash_or_number).await?;
        let mut stream = RlpStream::new();
        rlp_append(&block, &mut stream);
        Ok(hex::encode(stream.out().as_slice()))
    }

    pub async fn get_contract_height(
        &mut self,
        func_name: &str,
        contract_addr: Address,
    ) -> Result<u64> {
        let contract = Contract::from_json(
            self.client.eth(),
            contract_addr,
            include_bytes!("ckb_chain_abi.json"),
        )
        .map_err(|e| anyhow::anyhow!("failed to instantiate contract by parse abi: {}", e))?;
        let result = contract.query(func_name, (), None, Options::default(), None);
        let height: u64 = result.await?;
        info!("client contract header number : {:?}", height);
        Ok(height)
    }
    pub async fn get_mock_data(&mut self, contract_addr: Address) -> Result<Bytes> {
        let contract = Contract::from_json(
            self.client.eth(),
            contract_addr,
            include_bytes!("ckb_chain_abi.json"),
        )
        .map_err(|e| anyhow::anyhow!("failed to instantiate contract by parse abi: {}", e))?;
        let result = contract.query("mockHeaders", (), None, Options::default(), None);
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

        let result = contract.query(
            "getHeadersByNumber",
            Uint::from(block_number),
            None,
            Options::default(),
            None,
        );

        let header_hashes: Vec<FixedBytes> = result.await?;

        for hash in header_hashes {
            info!(
                "contact block {:?} header hash : {:?}",
                block_number,
                hex::encode(hash.as_slice())
            );
            if hash.as_slice() == latest_header_hash.as_bytes() {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

pub fn transfer_to_header(block: &Block<H256>) -> BlockHeader {
    BlockHeader {
        hash: block.hash,
        parent_hash: block.parent_hash,
        uncles_hash: block.uncles_hash,
        author: block.author,
        state_root: block.state_root,
        transactions_root: block.transactions_root,
        receipts_root: block.receipts_root,
        number: block.number,
        gas_used: block.gas_used,
        gas_limit: block.gas_limit,
        extra_data: block.extra_data.clone(),
        logs_bloom: block.logs_bloom.unwrap(),
        timestamp: block.timestamp,
        difficulty: block.difficulty,
        mix_hash: block.mix_hash,
        nonce: block.nonce,
    }
}

pub fn convert_to_header_rlp(block: &Block<H256>) -> Result<String> {
    let mut stream = RlpStream::new();
    rlp_append(&block, &mut stream);
    Ok(hex::encode(stream.out().as_slice()))
}

pub fn make_transaction(
    to: H160,
    nonce: U256,
    data: Vec<u8>,
    gas_price: U256,
    eth_value: U256,
) -> RawTransaction {
    RawTransaction {
        nonce: convert_u256(nonce),
        to: Some(convert_account(to)),
        value: eth_value,
        gas_price,
        gas: U256::from(MAX_GAS_LIMIT),
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

/// Gets the public address of a private key.
fn secret_key_address(key: &SecretKey) -> Address {
    let secp = Secp256k1::signing_only();
    let public_key = PublicKey::from_secret_key(&secp, key);
    public_key_address(&public_key)
}

fn public_key_address(public_key: &PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();

    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    Address::from_slice(&hash[12..])
}

/// Compute the Keccak-256 hash of input bytes.
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
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

pub fn decode_block_header(serialized: &Rlp) -> Result<BlockHeader, DecoderError> {
    let block_header = BlockHeader {
        parent_hash: serialized.val_at(0)?,
        uncles_hash: serialized.val_at(1)?,
        author: serialized.val_at(2)?,
        state_root: serialized.val_at(3)?,
        transactions_root: serialized.val_at(4)?,
        receipts_root: serialized.val_at(5)?,
        logs_bloom: serialized.val_at(6)?,
        difficulty: serialized.val_at(7)?,
        number: Some(serialized.val_at(8)?),
        gas_limit: serialized.val_at(9)?,
        gas_used: serialized.val_at(10)?,
        timestamp: serialized.val_at(11)?,
        extra_data: Bytes::from(serialized.as_raw()),
        mix_hash: Some(serialized.val_at(13)?),
        nonce: Some(serialized.val_at(14)?),
        hash: Some(my_keccak256(serialized.as_raw()).into()),
    };

    Ok(block_header)
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

pub fn convert_hex_to_h256(hex: String) -> Result<H256> {
    let bytes = strip_hex_prefix(&hex).and_then(decode_hex)?;
    Ok(H256::from_slice(&bytes))
}

pub fn strip_hex_prefix(prefixed_hex: &str) -> Result<String> {
    let res = str::replace(prefixed_hex, "0x", "");
    match res.len() % 2 {
        0 => Ok(res),
        _ => left_pad_with_zero(&res),
    }
}

fn left_pad_with_zero(string: &str) -> Result<String> {
    Ok(format!("0{}", string))
}

pub fn decode_hex(hex_to_decode: String) -> Result<Vec<u8>> {
    Ok(hex::decode(hex_to_decode)?)
}

pub fn build_lock_token_payload(data: &[Token]) -> Result<ethabi::Bytes> {
    let function = Function {
        name: "lockToken".to_owned(),
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
                name: "bridgeFee".to_owned(),
                kind: ParamType::Uint(256),
            },
            Param {
                name: "recipientLockscript".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "replayResistOutpoint".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "sudtExtraData".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    Ok(function.encode_input(data)?)
}

pub fn build_lock_eth_payload(data: &[Token]) -> Result<ethabi::Bytes> {
    let function = Function {
        name: "lockETH".to_owned(),
        inputs: vec![
            Param {
                name: "bridgeFee".to_owned(),
                kind: ParamType::Uint(256),
            },
            Param {
                name: "recipientLockscript".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "replayResistOutpoint".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "sudtExtraData".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    Ok(function.encode_input(data)?)
}

pub fn rlp_transaction(tx: &RawTransaction) -> String {
    let mut s = RlpStream::new();
    s.append(&tx.nonce);
    s.append(&tx.gas_price);
    s.append(&tx.gas);
    if let Some(ref t) = tx.to {
        s.append(t);
    } else {
        s.append(&vec![]);
    }
    s.append(&tx.value);
    s.append(&tx.data);
    hex::encode(s.out().as_slice())
}

#[tokio::test]
async fn test_get_block() {
    // use cmd_lib::run_fun;
    // use serde_json::{Result, Value};
    // use web3::types::{BlockNumber, U64};
    // let mut client = Web3Client::new(String::from(
    //     "https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2",
    // ));
    // let res = client.get_header_rlp(U64::from(3).into()).await;
    // println!("{:?}", res);
    // let header_rlp = format!("0x{}", res.unwrap());
    // println!("{:?}", header_rlp);
    // let path = String::from("/tmp/data.json");
    // let cmd = format!("pwd > {}", path);
    // println!("{}", cmd);
    // let hash = "0x731bc2fc859789d4190175281a43a446b6cc34c55bc8736eb793b4362803a19c";
    // let idx = 2;
    // let url = "https://ropsten.infura.io/v3/48be8feb3f9c46c397ceae02a0dbc7ae";
    // let res = run_fun! {
    // node /Users/leon/dev/rust/leon/eth-proof/index proof --hash ${hash} --index ${idx} --network ${url}}
    // .unwrap();
    //
    // println!("{:?}", res);
}
