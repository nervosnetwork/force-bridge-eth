use anyhow::Result;
use secp256k1::{Message, Secp256k1, SecretKey};
use web3::transports::Http;
use web3::types::{Block, BlockId, H256};
use web3::Web3;

pub fn get_secret_key(path: &str) -> Result<secp256k1::SecretKey> {
    let content = std::fs::read_to_string(path)?;
    let privkey_string = content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("File is empty"))?
        .to_string();
    let privkey_bytes = hex::decode(clear_0x(privkey_string.as_str()))?;
    Ok(secp256k1::SecretKey::from_slice(&privkey_bytes)?)
}

pub fn get_msg_signature(msg_hash: &[u8], eth_key: SecretKey) -> Result<Vec<u8>> {
    let secp = Secp256k1::signing_only();
    let message = Message::from_slice(msg_hash)?;

    let (recovery, sig_bytes) = secp
        .sign_recoverable(&message, &eth_key)
        .serialize_compact();

    let sig_v = recovery.to_i32() as u64 + 27;
    let mut signature = sig_bytes.to_vec();
    signature.push(sig_v as u8);
    Ok(signature)
}

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

pub struct Web3Client {
    _url: String,
    client: Web3<Http>,
}

impl Web3Client {
    pub fn new(url: String) -> Web3Client {
        let client = {
            let transport = web3::transports::Http::new(url.as_str()).expect("new transport");
            web3::Web3::new(transport)
        };
        Web3Client { _url: url, client }
    }

    pub async fn get_block(&mut self, hash_or_number: BlockId) -> Result<Block<H256>> {
        let block = self.client.eth().block(hash_or_number).await?;
        match block {
            Some(block) => Ok(block),
            None => anyhow::bail!("the block is not exist."),
        }
    }
}
