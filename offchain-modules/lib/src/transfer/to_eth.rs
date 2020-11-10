use crate::util::ckb_util::{covert_to_h256, make_ckb_transaction};
use anyhow::anyhow;
use anyhow::Result;
use ckb_sdk::rpc::{BlockView, TransactionView};
use ckb_sdk::{AddressPayload, HttpRpcClient, SECP256K1};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Entity, Pack, Unpack};
use ckb_types::utilities::CBMT;
use ckb_types::{packed, H256};
use ethabi::{Function, Param, ParamType};
use force_sdk::tx_helper::sign;
use force_sdk::util::{parse_privkey_path, send_tx_sync};
use log::debug;
use serde::export::Clone;
use serde::{Deserialize, Serialize};

// tx_merkle_index == index in transactions merkle tree of the block
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct CkbTxProof {
    pub tx_merkle_index: u16,
    pub block_number: u64,
    pub block_hash: H256,
    pub tx_hash: H256,
    pub witnesses_root: H256,
    pub lemmas: Vec<H256>,
}

pub fn burn(private_key: String, rpc_url: String) -> Result<String> {
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let from_privkey = parse_privkey_path(&private_key)?;
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    let from_lockscript = Script::from(&address_payload);
    let unsigned_tx = make_ckb_transaction(from_lockscript)?;
    let tx = sign(unsigned_tx, &mut rpc_client, &from_privkey)
        .map_err(|e| anyhow!("failed to sign tx : {}", e))?;
    log::info!(
        "tx: \n{}",
        serde_json::to_string_pretty(&ckb_jsonrpc_types::TransactionView::from(tx.clone()))?
    );
    send_tx_sync(&mut rpc_client, &tx, 60).map_err(|e| anyhow::anyhow!(e))?;
    let cell_typescript = tx
        .output(0)
        .ok_or_else(|| anyhow::anyhow!("first output cell is none"))?
        .type_()
        .to_opt();
    let cell_script = match cell_typescript {
        Some(script) => hex::encode(script.as_slice()),
        None => "".to_owned(),
    };
    let print_res = serde_json::json!({
        "tx_hash": hex::encode(tx.hash().as_slice()),
        "cell_typescript": cell_script,
    });
    debug!("{}", serde_json::to_string_pretty(&print_res)?);
    Ok(hex::encode(tx.hash().as_slice()))
}

pub fn unlock() -> Result<()> {
    todo!()
}

pub fn get_add_ckb_headers_func() -> Function {
    //TODO : addHeader is mock function for test feature which set header data in eth contract
    Function {
        name: "addHeaders".to_owned(),
        inputs: vec![Param {
            name: "data".to_owned(),
            kind: ParamType::Bytes,
        }],
        outputs: vec![],
        constant: false,
    }
}

pub fn parse_ckb_proof(tx_hash_str: &str, rpc_url: String) -> Result<CkbTxProof> {
    let tx_hash = covert_to_h256(tx_hash_str)?;
    let mut rpc_client = HttpRpcClient::new(rpc_url);
    let retrieved_block_hash = rpc_client
        .get_transaction(tx_hash.clone())
        .map_err(|e| anyhow!("failed to get ckb tx: {}", e))?
        .ok_or_else(|| anyhow!("Transaction {:#x} not exists", tx_hash))?
        .tx_status
        .block_hash
        .ok_or_else(|| anyhow!("Transaction {:#x} not yet in block", tx_hash))?;

    let retrieved_block = rpc_client
        .get_block(retrieved_block_hash.clone())
        .map_err(|e| anyhow::anyhow!("failed to get ckb block: {}", e))?
        .ok_or_else(|| anyhow!("block is none"))?;

    let tx_index = get_tx_index(&tx_hash, &retrieved_block)
        .ok_or_else(|| anyhow!("tx_hash not in retrieved_block"))? as u32;
    let tx_indices = vec![tx_index];
    debug!("tx index: {}", tx_index);
    let tx_num = retrieved_block.transactions.len();
    debug!("retrieved block hash {:?}", retrieved_block_hash);
    debug!("retrieved header hash {:?}", retrieved_block.header.hash);

    let proof = CBMT::build_merkle_proof(
        &retrieved_block
            .transactions
            .iter()
            .map(|tx| tx.hash.pack())
            .collect::<Vec<_>>(),
        &tx_indices,
    )
    .ok_or_else(|| anyhow!("build proof with verified inputs should be OK"))?;

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
            .map(|lemma| Unpack::<H256>::unpack(lemma))
            .collect(),
    })
}

pub fn get_tx_index(tx_hash: &H256, block: &BlockView) -> Option<usize> {
    block.transactions.iter().position(|tx| &tx.hash == tx_hash)
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
