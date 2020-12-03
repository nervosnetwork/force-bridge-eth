use crate::util::ckb_tx_generator::{Generator, CONFIRM};
use crate::util::ckb_types::CkbTxProof;
use crate::util::ckb_util::{covert_to_h256, parse_privkey, parse_privkey_path};
use crate::util::config::ForceConfig;
use crate::util::eth_util::{convert_eth_address, parse_private_key, Web3Client};
use crate::util::generated::ckb_tx_proof;
use anyhow::anyhow;
use anyhow::Result;
use ckb_sdk::rpc::{BlockView, TransactionView};
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, HumanCapacity, NetworkType};
use ckb_types::packed::{Byte32, Script};
use ckb_types::prelude::{Entity, Pack, Unpack};
use ckb_types::utilities::CBMT;
use ckb_types::{packed, H256};
use ethabi::{Function, Param, ParamType, Token};
use ethereum_types::U256;
use force_sdk::util::ensure_indexer_sync;
use log::{debug, info};
use serde::export::Clone;
use std::str::FromStr;
use web3::types::H160;

#[allow(clippy::too_many_arguments)]
pub async fn init_light_client(
    config_path: String,
    network: Option<String>,
    key_path: String,
    height: Option<u64>,
    finalized_gc_threshold: u64,
    canonical_gc_threshold: u64,
    gas_price: u64,
    wait: bool,
) -> Result<String> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let eth_ckb_chain_addr = convert_eth_address(
        &force_config
            .deployed_contracts
            .as_ref()
            .expect("contracts deployed")
            .eth_ckb_chain_addr,
    )?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let eth_rpc_url = force_config.get_ethereum_rpc_url(&network)?;
    let indexer_url = force_config.get_ckb_indexer_url(&network)?;
    let eth_private_key = parse_private_key(&key_path, &force_config, &network)?;
    let mut ckb_client = Generator::new(ckb_rpc_url, indexer_url, Default::default())
        .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
    let mut web3_client = Web3Client::new(eth_rpc_url);

    let height = if let Some(height) = height {
        height
    } else {
        let tip_number = ckb_client
            .rpc_client
            .get_tip_block_number()
            .map_err(|e| anyhow::anyhow!("failed to get tip number: {}", e))?;
        if tip_number <= CONFIRM as u64 {
            tip_number
        } else {
            tip_number - CONFIRM as u64
        }
    };
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
            eth_private_key,
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
pub async fn burn(
    config_path: String,
    network: Option<String>,
    privkey_path: String,
    tx_fee: String,
    unlock_fee: u128,
    amount: u128,
    token_addr: String,
    receive_addr: String,
) -> Result<String> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let lock_contract_addr = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let indexer_url = force_config.get_ckb_indexer_url(&network)?;

    let token_addr = convert_eth_address(&token_addr)?;
    let receive_addr = convert_eth_address(&receive_addr)?;
    let mut generator = Generator::new(ckb_rpc_url, indexer_url, deployed_contracts.clone())
        .map_err(|e| anyhow!("failed to crate generator: {}", e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .await
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;

    let from_privkey = parse_privkey_path(&privkey_path, &force_config, &network)?;
    let from_lockscript = parse_privkey(&from_privkey);
    let from_addr_payload: AddressPayload = from_lockscript.clone().into();
    let from_addr = Address::new(NetworkType::Dev, from_addr_payload);
    log::info!("from_addr: {}", from_addr);
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
    generator
        .sign_and_send_transaction(unsigned_tx, from_privkey)
        .await
}

#[allow(clippy::never_loop)]
pub async fn wait_block_submit(
    eth_url: String,
    ckb_url: String,
    light_contract_addr: H160,
    tx_hash: String,
    lock_contract_addr: H160,
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
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
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
            .get_contract_height("latestBlockNumber", light_contract_addr)
            .await?;
        let confirm = web3_client
            .get_locker_contract_confirm("numConfirmations_", lock_contract_addr)
            .await?;
        info!(
            "client_block_number : {:?},ckb_height :{:?}, confirm :{:?}",
            client_block_number, ckb_height, confirm
        );
        if client_block_number < ckb_height + confirm {
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
            continue;
        }
        return Ok(());
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn unlock(
    config_path: String,
    network: Option<String>,
    key_path: String,
    to: String,
    tx_proof: String,
    raw_tx: String,
    gas_price: u64,
    wait: bool,
) -> Result<String> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let eth_url = force_config.get_ethereum_rpc_url(&network)?;
    let to = convert_eth_address(&to)?;
    let eth_private_key = parse_private_key(&key_path, &force_config, &network)?;
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
                name: "ckbTx".to_owned(),
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
            eth_private_key,
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
        .get_transaction(tx_hash.clone())
        .map_err(|e| anyhow!("failed to get tx {}, err: {}", &tx_hash, e))?
        .ok_or_else(|| anyhow!("failed to get tx : {}", &tx_hash))?
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
pub async fn transfer_sudt(
    config_path: String,
    network: Option<String>,
    privkey_path: String,
    to_addr: String,
    tx_fee: String,
    ckb_amount: String,
    transfer_amount: u128,
    token_addr: String,
) -> Result<String> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;

    let token_addr = convert_eth_address(&token_addr)?;
    let lock_contract_addr = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;
    let mut generator = Generator::new(ckb_rpc_url, ckb_indexer_url, deployed_contracts.clone())
        .map_err(|e| anyhow!(e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .await
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;

    let from_privkey = parse_privkey_path(&privkey_path, &force_config, &network)?;
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

    generator
        .sign_and_send_transaction(unsigned_tx, from_privkey)
        .await
}

pub async fn get_balance(
    config_path: String,
    network: Option<String>,
    address: String,
    token_addr: String,
) -> Result<u128> {
    let force_config = ForceConfig::new(config_path.as_str())?;
    let deployed_contracts = force_config
        .deployed_contracts
        .as_ref()
        .ok_or_else(|| anyhow!("contracts should be deployed"))?;
    let ckb_rpc_url = force_config.get_ckb_rpc_url(&network)?;
    let ckb_indexer_url = force_config.get_ckb_indexer_url(&network)?;
    let token_addr = convert_eth_address(&token_addr)?;
    let lock_contract_addr = convert_eth_address(&deployed_contracts.eth_token_locker_addr)?;

    let mut generator = Generator::new(ckb_rpc_url, ckb_indexer_url, deployed_contracts.clone())
        .map_err(|e| anyhow!(e))?;
    ensure_indexer_sync(&mut generator.rpc_client, &mut generator.indexer_client, 60)
        .await
        .map_err(|e| anyhow!("failed to ensure indexer sync : {}", e))?;
    let addr_lockscript: Script = Address::from_str(&address)
        .map_err(|err| anyhow!(err))?
        .payload()
        .into();
    let balance = generator
        .get_sudt_balance(addr_lockscript, token_addr, lock_contract_addr)
        .map_err(|e| anyhow!("failed to get balance of {:?}  : {}", address, e))?;
    Ok(balance)
}
