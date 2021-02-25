use anyhow::{anyhow, Result};
use secp256k1::{Message, Secp256k1, SecretKey};

pub fn get_secret_key(privkey_string: &str) -> Result<secp256k1::SecretKey> {
    let privkey_bytes = hex::decode(clear_0x(privkey_string))?;
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
