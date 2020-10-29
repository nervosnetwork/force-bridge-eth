use crate::util::ckb_util::{covert_to_h256, make_ckb_transaction};
use crate::util::eth_util::Web3Client;
use anyhow::Result;
use ckb_sdk::rpc::{BlockView, TransactionView};
use ckb_sdk::{AddressPayload, HttpRpcClient, SECP256K1};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Entity, Pack, Unpack};
use ckb_types::utilities::CBMT;
use ckb_types::{packed, H256 as ckb_H256};
use ethabi::{Function, Param, ParamType, Token};
use force_sdk::tx_helper::sign;
use force_sdk::util::{parse_privkey_path, send_tx_sync};
use log::debug;
use serde::export::Clone;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use web3::types::{H160, H256 as web3_H256, U256};

pub fn burn(private_key: String, rpc_url: String) -> Result<String> {
    let mut rpc_client = HttpRpcClient::new(rpc_url);
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
    debug!("{}", serde_json::to_string_pretty(&print_res).unwrap());
    Ok(hex::encode(tx.hash().as_slice()))
}

pub fn unlock() -> Result<()> {
    todo!()
}

pub async fn relay_ckb_headers(
    from: H160,
    to: H160,
    url: String,
    key_path: String,
    headers: Vec<u8>,
) -> web3_H256 {
    let mut rpc_client = Web3Client::new(url);
    let function = Function {
        name: "addHeader".to_owned(),
        inputs: vec![Param {
            name: "_data".to_owned(),
            kind: ParamType::Bytes,
        }],
        outputs: vec![],
        constant: false,
    };
    let data = function.encode_input(&[Token::Bytes(headers)]).unwrap();
    debug!("data : {:?}", hex::encode(data.as_slice()));
    rpc_client
        .send_transaction(from, to, key_path, data, U256::from(0))
        .await
        .expect("invalid tx hash")
}

pub fn parse_ckb_proof(tx_hash_str: &str, rpc_url: String) -> Result<CkbTxProof, String> {
    let tx_hash = covert_to_h256(tx_hash_str)?;
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let retrieved_block_hash;
    let retrieved_block;
    let mut tx_indices = HashSet::new();
    let tx_index;
    match rpc_client.get_transaction(tx_hash.clone())? {
        Some(tx_with_status) => {
            retrieved_block_hash = tx_with_status.tx_status.block_hash;
            retrieved_block = rpc_client
                .get_block(retrieved_block_hash.clone().expect("tx_block_hash is none"))?
                .expect("block is none");

            tx_index = get_tx_index(&tx_hash, &retrieved_block)
                .expect("tx_hash not in retrieved_block") as u32;
            dbg!(tx_index);
            if !tx_indices.insert(tx_index) {
                return Err(format!("Duplicated tx_hash {:#x}", tx_hash));
            }
        }
        None => {
            return Err(format!("Transaction {:#x} not yet in block", tx_hash));
        }
    }

    let tx_num = retrieved_block.transactions.len();
    let retrieved_block_hash = retrieved_block_hash.expect("checked len");
    dbg!(format!("{:#x}", retrieved_block_hash));
    dbg!(format!("{:#x}", retrieved_block.header.hash));

    let proof = CBMT::build_merkle_proof(
        &retrieved_block
            .transactions
            .iter()
            .map(|tx| tx.hash.pack())
            .collect::<Vec<_>>(),
        &tx_indices.into_iter().collect::<Vec<_>>(),
    )
    .expect("build proof with verified inputs should be OK");

    // tx_merkle_index means the tx index in transactions merkle tree of the block
    Ok(CkbTxProof {
        block_hash: retrieved_block_hash,
        block_number: retrieved_block.header.inner.number,
        tx_hash,
        tx_merkle_index: (tx_index + tx_num as u32 - 1) as u16,
        witnesses_root: calc_witnesses_root(retrieved_block.transactions).unpack(),
        lemmas: proof
            .lemmas()
            .iter()
            .map(|lemma| Unpack::<ckb_H256>::unpack(lemma))
            .collect(),
    })
}

// tx_merkle_index == index in transactions merkle tree of the block
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct CkbTxProof {
    pub tx_merkle_index: u16,
    pub block_number: u64,
    pub block_hash: ckb_H256,
    pub tx_hash: ckb_H256,
    pub witnesses_root: ckb_H256,
    pub lemmas: Vec<ckb_H256>,
}

pub fn calc_witnesses_root(transactions: Vec<TransactionView>) -> Byte32 {
    let leaves = transactions
        .iter()
        .map(|tx| {
            let tx: packed::Transaction = tx.clone().inner.into();
            tx.calc_witness_hash()
        })
        .collect::<Vec<Byte32>>();

    CBMT::build_merkle_root(leaves.as_ref())
}
pub fn get_tx_index(tx_hash: &ckb_H256, block: &BlockView) -> Option<usize> {
    block.transactions.iter().position(|tx| &tx.hash == tx_hash)
}
