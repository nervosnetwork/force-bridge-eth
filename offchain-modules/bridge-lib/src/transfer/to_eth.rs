use ckb_sdk::{ AddressPayload, HttpRpcClient, SECP256K1 };
use ckb_types::packed::Script;
use crate::util::ckb_util::make_ckb_transaction;
use ckb_types::prelude::Entity;
use anyhow::Result;
use bridge_ckb_sdk::util::{parse_privkey_path, send_tx_sync};
use bridge_ckb_sdk::tx_helper::sign;

pub fn burn(private_key: String, rpc_url: String) -> Result<String>{
    let mut rpc_client = HttpRpcClient::new(rpc_url.clone());
    let from_privkey = parse_privkey_path(private_key.as_str())?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);
    let unsigned_tx = make_ckb_transaction(from_lockscript).unwrap();
    let tx = sign(unsigned_tx, &mut rpc_client, &from_privkey).unwrap();
    log::info!(
        "tx: \n{}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))
            .unwrap()
    );
    send_tx_sync(&mut rpc_client, &tx, 60).map_err(|e| anyhow::anyhow!(e))?;
    let cell_typescript = tx.output(0).unwrap().type_().to_opt();
    let cell_script = match cell_typescript {
        Some(script) => hex::encode(script.as_slice()),
        None => "".to_owned(),
    };
    let print_res = serde_json::json!({
        "tx_hash": hex::encode(tx.hash().as_slice()),
        "cell_typescript": cell_script,
    });
    println!("{}", serde_json::to_string_pretty(&print_res).unwrap());
    Ok(hex::encode(tx.hash().as_slice()))
}

pub fn parse_ckb_proof() -> Result<()>{
    todo!()
}

pub fn unlock() -> Result<()> {
    todo!()
}

