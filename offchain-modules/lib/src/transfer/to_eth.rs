use crate::util::ckb_types::CkbTxProof;
use crate::util::ckb_util::{covert_to_h256, parse_privkey, Generator};
use crate::util::eth_util::Web3Client;
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
use ethabi::{Function, Param, ParamType, Token};
use ethereum_types::U256;
use force_sdk::util::{ensure_indexer_sync, parse_privkey_path};
use log::{debug, info};
use serde::export::Clone;
use std::str::FromStr;
use web3::types::H160;

#[allow(clippy::too_many_arguments)]
pub async fn init_light_client(
    ckb_rpc_url: String,
    indexer_url: String,
    eth_rpc_url: String,
    height: u64,
    finalized_gc_threshold: u64,
    canonical_gc_threshold: u64,
    gas_price: u64,
    eth_ckb_chain_addr: H160,
    key_path: String,
    wait: bool,
) -> Result<String> {
    let mut ckb_client = Generator::new(ckb_rpc_url, indexer_url, Default::default())
        .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
    let mut web3_client = Web3Client::new(eth_rpc_url);

    let header = ckb_client
        .rpc_client
        .get_header_by_number(height)
        .map_err(|e| anyhow::anyhow!("failed to get header: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("failed to get header which is none"))?;

    let mol_header: packed::Header = header.clone().inner.into();

    let init_func = get_init_ckb_headers_func();
    let init_header_abi = init_func.encode_input(&[
        Token::Bytes(Vec::from(mol_header.raw().as_slice())),
        Token::FixedBytes(Vec::from(header.hash.as_bytes())),
        Token::Uint(U256::from(finalized_gc_threshold)),
        Token::Uint(U256::from(canonical_gc_threshold)),
    ])?;
    let res = web3_client
        .send_transaction(
            eth_ckb_chain_addr,
            key_path,
            init_header_abi,
            U256::from(gas_price),
            U256::zero(),
            wait,
        )
        .await?;
    let tx_hash = hex::encode(res);
    Ok(tx_hash)
}

#[allow(clippy::too_many_arguments)]
pub fn burn(
    privkey_path: String,
    rpc_url: String,
    indexer_url: String,
    config_path: &str,
    tx_fee: String,
    unlock_fee: u128,
    amount: u128,
    token_addr: H160,
    receive_addr: H160,
    lock_contract_addr: H160,
) -> Result<String> {
    let settings = Settings::new(config_path)?;
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
        .burn(
            tx_fee,
            from_lockscript,
            unlock_fee,
            amount,
            token_addr,
            lock_contract_addr,
            receive_addr,
        )
        .map_err(|e| anyhow!("failed to build burn tx : {}", e))?;

    generator.sign_and_send_transaction(unsigned_tx, from_privkey)
}

#[allow(clippy::never_loop)]
pub async fn wait_block_submit(
    eth_url: String,
    ckb_url: String,
    contract_addr: H160,
    tx_hash: String,
) -> Result<()> {
    let mut ckb_client = HttpRpcClient::new(ckb_url);
    let hash = covert_to_h256(&tx_hash)?;
    let block_hash;

    loop {
        let block_hash_opt = ckb_client
            .get_transaction(hash.clone())
            .map_err(|err| anyhow!(err))?
            .ok_or_else(|| anyhow!("tx is none"))?
            .tx_status
            .block_hash;
        match block_hash_opt {
            Some(hash) => {
                block_hash = hash;
                break;
            }
            None => {
                info!("the transaction is not committed yet");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    let ckb_height = ckb_client
        .get_block(block_hash)
        .map_err(|err| anyhow!(err))?
        .ok_or_else(|| anyhow!("block is none"))?
        .header
        .inner
        .number;
    let mut web3_client = Web3Client::new(eth_url);
    loop {
        let client_block_number = web3_client
            .get_contract_height("latestBlockNumber", contract_addr)
            .await?;
        info!(
            "client_block_number : {:?},ckb_height :{:?}",
            client_block_number, ckb_height
        );
        if client_block_number < ckb_height {
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }
        return Ok(());
    }
}

pub async fn unlock(
    to: H160,
    key_path: String,
    tx_proof: String,
    raw_tx: String,
    eth_url: String,
    gas_price: u64,
    wait: bool,
) -> Result<String> {
    let mut rpc_client = Web3Client::new(eth_url);
    let proof = hex::decode(tx_proof).map_err(|err| anyhow!(err))?;
    let tx_info = hex::decode(raw_tx).map_err(|err| anyhow!(err))?;

    let function = Function {
        name: "unlockToken".to_owned(),
        inputs: vec![
            Param {
                name: "ckbTxProof".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "txInfo".to_owned(),
                kind: ParamType::Bytes,
            },
        ],
        outputs: vec![],
        constant: false,
    };
    let tokens = [Token::Bytes(proof), Token::Bytes(tx_info)];
    let input_data = function.encode_input(&tokens)?;
    let res = rpc_client
        .send_transaction(
            to,
            key_path,
            input_data,
            U256::from(gas_price),
            U256::from(0),
            wait,
        )
        .await?;
    let tx_hash = hex::encode(res);
    Ok(tx_hash)
}

pub fn get_add_ckb_headers_func() -> Function {
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

pub fn get_init_ckb_headers_func() -> Function {
    Function {
        name: "initWithHeader".to_owned(),
        inputs: vec![
            Param {
                name: "data".to_owned(),
                kind: ParamType::Bytes,
            },
            Param {
                name: "blockHash".to_owned(),
                kind: ParamType::FixedBytes(32),
            },
            Param {
                name: "finalizedGcThreshold".to_owned(),
                kind: ParamType::Uint(64),
            },
            Param {
                name: "canonicalGcThreshold".to_owned(),
                kind: ParamType::Uint(64),
            },
        ],
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
    ckb_amount: String,
    transfer_amount: u128,
    token_addr: H160,
    lock_contract_addr: H160,
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
    let ckb_amount: u64 = HumanCapacity::from_str(&ckb_amount)
        .map_err(|e| anyhow!(e))?
        .into();

    let unsigned_tx = generator
        .transfer_sudt(
            lock_contract_addr,
            token_addr,
            from_lockscript,
            to_lockscript,
            transfer_amount,
            ckb_amount,
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
    lock_contract_addr: H160,
) -> Result<u128> {
    let settings = Settings::new(&config_path)?;
    let mut generator = Generator::new(rpc_url, indexer_url, settings).map_err(|e| anyhow!(e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
    let balance = generator
        .get_sudt_balance(address.clone(), token_addr, lock_contract_addr)
        .map_err(|e| anyhow!("failed to get balance of {:?}  : {}", address, e))?;
    Ok(balance)
}
