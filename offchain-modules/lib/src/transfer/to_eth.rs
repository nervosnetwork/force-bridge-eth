use crate::util::ckb_types::CkbTxProof;
use crate::util::ckb_util::{covert_to_h256, parse_privkey, Generator};
use crate::util::generated::ckb_tx_proof;
use crate::util::settings::Settings;
use anyhow::anyhow;
use anyhow::Result;
use ckb_sdk::rpc::{BlockView, TransactionView};
use ckb_sdk::{Address, HttpRpcClient, HumanCapacity};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Entity, Pack, Unpack};
use ckb_types::utilities::CBMT;
use ckb_types::{packed, H256};
use ethabi::{Function, Param, ParamType};
use force_sdk::util::{ensure_indexer_sync, parse_privkey_path};
use log::{debug, info};
use serde::export::Clone;
use std::str::FromStr;
use web3::types::H160;

#[allow(clippy::too_many_arguments)]
pub fn burn(
    privkey_path: String,
    rpc_url: String,
    indexer_url: String,
    config_path: String,
    tx_fee: String,
    amount: u128,
    token_addr: H160,
    receive_addr: H160,
) -> Result<String> {
    let settings = Settings::new(&config_path)?;
    let mut generator = Generator::new(rpc_url, indexer_url, settings)
        .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;

    let from_privkey = parse_privkey_path(&privkey_path)?;
    let from_lockscript = parse_privkey(&from_privkey);

    let tx_fee: u64 = HumanCapacity::from_str(&tx_fee)
        .map_err(|e| anyhow!(e))?
        .into();

    let unsigned_tx = generator
        .burn(tx_fee, from_lockscript, amount, token_addr, receive_addr)
        .map_err(|e| anyhow!("failed to build burn tx : {}", e))?;

    generator.sign_and_send_transaction(unsigned_tx, from_privkey)
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

pub fn get_ckb_proof_info(tx_hash_str: &str, rpc_url: String) -> Result<(String, String)> {
    let tx_hash = covert_to_h256(tx_hash_str)?;
    let mut rpc_client = HttpRpcClient::new(rpc_url.clone());
    let tx: packed::Transaction = rpc_client
        .get_transaction(tx_hash)
        .map_err(|e| anyhow!("failed to sign tx : {}", e))?
        .ok_or_else(|| anyhow!("failed to sign tx : {}"))?
        .transaction
        .inner
        .into();

    let mol_hex_tx = hex::encode(tx.raw().as_slice());
    info!("mol hex raw tx : {:?} ", mol_hex_tx);

    let ckb_tx_proof = parse_ckb_proof(tx_hash_str, rpc_url)?;
    let mol_tx_proof: ckb_tx_proof::CkbTxProof = ckb_tx_proof.into();
    let mol_hex_header = hex::encode(mol_tx_proof.as_bytes().as_ref());
    info!("mol hex header: {:?} ", mol_hex_header);
    Ok((mol_hex_header, mol_hex_tx))
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
        .map_err(|e| anyhow!("failed to get ckb block: {}", e))?
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

#[allow(clippy::too_many_arguments)]
pub fn transfer_sudt(
    privkey_path: String,
    rpc_url: String,
    indexer_url: String,
    config_path: String,
    to_addr: String,
    tx_fee: String,
    transfer_amount: u128,
    token_addr: H160,
) -> Result<String> {
    let settings = Settings::new(&config_path)?;
    let mut generator = Generator::new(rpc_url, indexer_url, settings).map_err(|e| anyhow!(e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;

    let from_privkey = parse_privkey_path(&privkey_path)?;
    let from_lockscript = parse_privkey(&from_privkey);

    let to_lockscript: Script = Address::from_str(&to_addr)
        .map_err(|e| anyhow!("failed to covert address  : {}", e))?
        .payload()
        .into();

    let tx_fee: u64 = HumanCapacity::from_str(&tx_fee)
        .map_err(|e| anyhow!(e))?
        .into();

    let unsigned_tx = generator
        .transfer_sudt(
            from_lockscript,
            token_addr,
            to_lockscript,
            transfer_amount,
            200,
            tx_fee,
        )
        .map_err(|e| anyhow!("failed to build transfer token tx: {}", e))?;

    generator.sign_and_send_transaction(unsigned_tx, from_privkey)
}

pub fn get_balance(
    rpc_url: String,
    indexer_url: String,
    config_path: String,
    address: String,
    token_addr: H160,
) -> Result<u128> {
    let settings = Settings::new(&config_path)?;
    let mut generator = Generator::new(rpc_url, indexer_url, settings).map_err(|e| anyhow!(e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
    let balance = generator
        .get_sudt_balance(address.clone(), token_addr)
        .map_err(|e| anyhow!("failed to get balance of {:?}  : {}", address, e))?;
    Ok(balance)
}
