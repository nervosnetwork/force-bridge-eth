use crate::util::ckb_tx_generator::{Generator, CONFIRM};
use crate::util::config::{DeployedContracts, ForceConfig, OutpointConf};
use crate::util::eth_proof_helper::{DoubleNodeWithMerkleProofJson, Witness};
use crate::util::eth_util::{convert_to_header_rlp, decode_block_header};
use anyhow::{anyhow, bail, Result};
use ckb_sdk::{Address, AddressPayload, SECP256K1};
use ckb_types::packed::{Byte, ScriptReader, WitnessArgs};
use ckb_types::prelude::{Builder, Entity, Pack, Reader};
use ckb_types::{
    bytes::Bytes,
    packed::{Byte32, OutPoint, Script},
    H256,
};
use ethereum_types::H160;
use faster_hex::hex_decode;
use force_eth_types::eth_recipient_cell::ETHAddress;
use force_eth_types::generated::basic::BytesVec;
use force_eth_types::generated::eth_bridge_lock_cell::ETHBridgeLockArgs;
use force_eth_types::generated::eth_header_cell::{
    DoubleNodeWithMerkleProof, ETHHeaderCellDataReader, ETHHeaderCellMerkleDataReader,
    ETHHeaderInfo, ETHHeaderInfoReader, MerkleProof,
};
use force_eth_types::generated::{basic, witness};
use force_sdk::cell_collector::get_live_cell_by_typescript;
use rlp::Rlp;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::ops::Add;
use std::str::FromStr;
use web3::types::{Block, BlockHeader};

pub fn get_secret_key(privkey_string: &str) -> Result<secp256k1::SecretKey> {
    let privkey_bytes = hex::decode(clear_0x(privkey_string))?;
    Ok(secp256k1::SecretKey::from_slice(&privkey_bytes)?)
}

pub fn parse_privkey_path(
    path: &str,
    config: &ForceConfig,
    network: &Option<String>,
) -> Result<secp256k1::SecretKey> {
    let privkey_string = if let Ok(index) = path.parse::<usize>() {
        let priv_keys = config.get_ckb_private_keys(network)?;
        priv_keys[index].clone()
    } else {
        let content = std::fs::read_to_string(path)?;
        content
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("File is empty"))?
            .to_string()
    };
    let privkey_bytes = hex::decode(clear_0x(privkey_string.as_str()))?;
    Ok(secp256k1::SecretKey::from_slice(&privkey_bytes)?)
}

pub fn build_merkle_proofs(witness: &Witness) -> Result<MerkleProof> {
    let proof_vec = &witness.merkle_proof;
    let mut proof_json_vec = vec![];
    for item in proof_vec {
        let dag_nodes = &item.dag_nodes;
        let mut dag_nodes_string = vec![];
        for node in dag_nodes {
            dag_nodes_string.push(hex::encode(node.0));
        }
        let proofs = &item.proof;
        let mut proof_string = vec![];
        for proof in proofs {
            proof_string.push(hex::encode(proof.0));
        }
        proof_json_vec.push(DoubleNodeWithMerkleProofJson {
            dag_nodes: dag_nodes_string,
            proof: proof_string,
        })
    }
    let mut merkle_proofs: Vec<DoubleNodeWithMerkleProof> = vec![];
    for item in proof_json_vec {
        let p: DoubleNodeWithMerkleProof = item.clone().try_into().unwrap();
        merkle_proofs.push(p);
    }
    let mut proofs = vec![];
    for item in merkle_proofs {
        proofs.push(basic::Bytes::from(item.as_slice().to_vec()));
    }
    let result = MerkleProof::new_builder().set(proofs).build();
    Ok(result)
}

pub fn covert_to_h256(mut tx_hash: &str) -> Result<H256> {
    if tx_hash.starts_with("0x") || tx_hash.starts_with("0X") {
        tx_hash = &tx_hash[2..];
    }
    if tx_hash.len() % 2 != 0 {
        bail!(format!("Invalid hex string length: {}", tx_hash.len()))
    }
    let mut bytes = vec![0u8; tx_hash.len() / 2];
    hex_decode(tx_hash.as_bytes(), &mut bytes)
        .map_err(|err| anyhow!("parse hex string failed: {:?}", err))?;
    H256::from_slice(&bytes).map_err(|e| anyhow!("failed to covert tx hash: {}", e))
}

pub fn get_sudt_type_script(
    deployed_contracts: &DeployedContracts,
    token_addr: H160,
    lock_contract_addr: H160,
) -> Result<Script> {
    let bridge_lockscript =
        create_bridge_lockscript(&deployed_contracts, &token_addr, &lock_contract_addr)?;
    // let lockscript_code_hash = hex::decode(&deployed_contracts.bridge_lockscript.code_hash)?;
    // let bridge_lockscript_code_hash =
    //     hex::decode(bridge_lock_code_hash).map_err(|err| anyhow!(err))?;
    // let bridge_lockscript = get_eth_bridge_lock_script(
    //     bridge_lockscript_code_hash.as_slice(),
    //     bridge_lock_hash_type,
    //     token_addr,
    //     lock_contract_addr,
    // )?;
    log::info!(
        "bridge lockscript: {}",
        serde_json::to_string(&ckb_jsonrpc_types::Script::from(bridge_lockscript.clone())).unwrap()
    );

    let sudt_typescript_code_hash = hex::decode(&deployed_contracts.sudt.code_hash)?;
    Ok(Script::new_builder()
        .code_hash(Byte32::from_slice(&sudt_typescript_code_hash).map_err(|err| anyhow!(err))?)
        .hash_type(deployed_contracts.sudt.hash_type.into())
        .args(bridge_lockscript.calc_script_hash().as_bytes().pack())
        .build())
}

pub fn get_eth_bridge_lock_script(
    bridge_lock_code_hash: &[u8],
    bridge_lockscript_hash_type: u8,
    token_addr: H160,
    lock_contract_addr: H160,
) -> Result<Script> {
    let args = ETHBridgeLockArgs::new_builder()
        .eth_contract_address(
            ETHAddress::try_from(lock_contract_addr.as_bytes().to_vec())
                .map_err(|err| anyhow!(err))?
                .get_address()
                .into(),
        )
        .eth_token_address(
            ETHAddress::try_from(token_addr.as_bytes().to_vec())
                .map_err(|err| anyhow!(err))?
                .get_address()
                .into(),
        )
        .build();

    Ok(Script::new_builder()
        .code_hash(Byte32::from_slice(bridge_lock_code_hash).map_err(|err| anyhow!(err))?)
        .hash_type(Byte::new(bridge_lockscript_hash_type))
        .args(args.as_bytes().pack())
        .build())
}

pub fn parse_privkey(privkey: &SecretKey) -> Script {
    let public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, privkey);
    let address_payload = AddressPayload::from_pubkey(&public_key);
    Script::from(&address_payload)
}

pub fn build_outpoint(outpoint_conf: OutpointConf) -> Result<OutPoint> {
    let outpoint = OutPoint::new_builder()
        .tx_hash(
            Byte32::from_slice(&hex::decode(outpoint_conf.tx_hash).map_err(|e| anyhow!(e))?)
                .map_err(|e| anyhow!(e))?,
        )
        .index(outpoint_conf.index.pack())
        .build();
    Ok(outpoint)
}

pub fn get_eth_client_tip_number(
    generator: &mut Generator,
    client_cell_script: String,
) -> Result<u64> {
    let script = parse_cell(client_cell_script.as_str())
        .map_err(|e| anyhow!("get typescript fail {:?}", e))?;
    let cell = get_live_cell_by_typescript(&mut generator.indexer_client, script)
        .map_err(|e| anyhow!("get live cell fail: {}", e))?
        .ok_or_else(|| anyhow!("eth header cell not exist"))?;
    // let (un_confirmed_headers, _) = parse_main_chain_headers(cell.output_data.as_bytes().to_vec())
    //     .map_err(|e| anyhow!("parse header data fail: {}", e))?;
    // let best_header = un_confirmed_headers
    //     .last()
    //     .ok_or_else(|| anyhow!("header is none"))?;
    // let best_number = best_header
    //     .number
    //     .ok_or_else(|| anyhow!("header number is none"))?
    //     .as_u64();
    let (_, latest_height, _) = parse_merkle_cell_data(cell.output_data.as_bytes().to_vec())?;
    Ok(latest_height)
}

pub fn parse_cell(cell: &str) -> Result<Script> {
    let cell_bytes =
        hex::decode(cell).map_err(|e| anyhow!("cell shoule be hex format, err: {}", e))?;
    ScriptReader::verify(&cell_bytes, false).map_err(|e| anyhow!("cell decoding err: {}", e))?;
    let cell_typescript = Script::new_unchecked(cell_bytes.into());
    Ok(cell_typescript)
}

pub fn build_lockscript_from_address(address: &str) -> Result<Script> {
    let recipient_lockscript = Script::from(
        Address::from_str(address)
            .map_err(|err| anyhow!(err))?
            .payload(),
    );
    Ok(recipient_lockscript)
}

#[allow(clippy::type_complexity)]
pub fn parse_main_raw_data(data: &Bytes) -> Result<(Vec<&[u8]>, Vec<&[u8]>)> {
    ETHHeaderCellDataReader::verify(data, false).map_err(|err| anyhow!(err))?;
    let chain_reader = ETHHeaderCellDataReader::new_unchecked(data);
    let main_reader = chain_reader.headers().main();
    let len = main_reader.len();
    let mut un_confirmed: Vec<&[u8]> = vec![];
    let mut confirmed: Vec<&[u8]> = vec![];
    for i in 0..len {
        let raw_data = main_reader.get_unchecked(i).raw_data();
        if (len - i) <= CONFIRM {
            un_confirmed.push(raw_data);
        } else {
            confirmed.push(main_reader.get_unchecked(i).raw_data());
        }
    }
    Ok((un_confirmed, confirmed))
}

pub fn parse_uncle_raw_data(data: &Bytes) -> Result<Vec<&[u8]>> {
    ETHHeaderCellDataReader::verify(data, false).map_err(|err| anyhow!(err))?;
    let chain_reader = ETHHeaderCellDataReader::new_unchecked(data);
    let uncle_reader = chain_reader.headers().uncle();
    let len = uncle_reader.len();
    let mut result = vec![];
    for i in 0..len {
        result.push(uncle_reader.get_unchecked(i).raw_data())
    }
    Ok(result)
}

pub fn parse_main_chain_headers(data: Vec<u8>) -> Result<(Vec<BlockHeader>, Vec<Vec<u8>>)> {
    ETHHeaderCellDataReader::verify(&data, false).map_err(|err| anyhow!(err))?;
    let chain_reader = ETHHeaderCellDataReader::new_unchecked(&data);
    let main_reader = chain_reader.headers().main();
    let mut un_confirmed = vec![];
    let mut confirmed = vec![];
    let len = main_reader.len();
    for i in (0..len).rev() {
        if (len - i) < CONFIRM {
            let header_raw = main_reader.get_unchecked(i).raw_data();
            ETHHeaderInfoReader::verify(&header_raw, false).map_err(|err| anyhow!(err))?;
            let header_info_header = ETHHeaderInfoReader::new_unchecked(header_raw);
            let rlp = Rlp::new(header_info_header.header().raw_data());
            let header: BlockHeader = decode_block_header(&rlp).map_err(|err| anyhow!(err))?;
            un_confirmed.push(header);
        } else {
            confirmed.push(main_reader.get_unchecked(i).raw_data().to_vec())
        }
    }
    un_confirmed.reverse();
    Ok((un_confirmed, confirmed))
}

pub fn parse_merkle_cell_data(data: Vec<u8>) -> Result<(u64, u64, [u8; 32])> {
    ETHHeaderCellMerkleDataReader::verify(&data, false).map_err(|err| anyhow!(err))?;
    let eth_cell_data_reader = ETHHeaderCellMerkleDataReader::new_unchecked(&data);

    let mut merkle_root = [0u8; 32];
    merkle_root.copy_from_slice(eth_cell_data_reader.merkle_root().raw_data());

    let mut last_cell_latest_height_raw = [0u8; 8];
    last_cell_latest_height_raw.copy_from_slice(eth_cell_data_reader.latest_height().raw_data());
    let last_cell_latest_height = u64::from_le_bytes(last_cell_latest_height_raw);

    let mut start_height_raw = [0u8; 8];
    start_height_raw.copy_from_slice(eth_cell_data_reader.start_height().raw_data());
    let start_height = u64::from_le_bytes(start_height_raw);

    Ok((start_height, last_cell_latest_height, merkle_root))
}

pub fn create_bridge_lockscript(
    deployed_contracts: &DeployedContracts,
    token: &H160,
    eth_address: &H160,
) -> Result<Script> {
    let cell_script = parse_cell(
        deployed_contracts
            .light_client_cell_script
            .cell_script
            .as_str(),
    )?;
    let lockscript_code_hash = hex::decode(&deployed_contracts.bridge_lockscript.code_hash)?;
    use force_eth_types::generated::basic::ETHAddress;
    let args = ETHBridgeLockArgs::new_builder()
        .eth_token_address(ETHAddress::from_slice(&token.as_bytes()).map_err(|err| anyhow!(err))?)
        .eth_contract_address(
            ETHAddress::from_slice(&eth_address.as_bytes()).map_err(|err| anyhow!(err))?,
        )
        .light_client_typescript_hash(force_eth_types::generated::basic::Byte32::from_slice(
            cell_script.calc_script_hash().raw_data().as_ref(),
        )?)
        .build();
    let lockscript = Script::new_builder()
        .code_hash(Byte32::from_slice(&lockscript_code_hash)?)
        .hash_type(deployed_contracts.bridge_lockscript.hash_type.into())
        .args(args.as_bytes().pack())
        .build();
    Ok(lockscript)
}

pub fn handle_unconfirmed_headers(
    input_tail_raw: &[u8],
    headers: &[Block<ethereum_types::H256>],
) -> Result<Vec<ETHHeaderInfo>> {
    let mut header_infos = vec![];
    ETHHeaderInfoReader::verify(&input_tail_raw, false).map_err(|err| anyhow!(err))?;
    let input_tail_reader = ETHHeaderInfoReader::new_unchecked(&input_tail_raw);
    let mut total_difficulty = to_u64(input_tail_reader.total_difficulty().raw_data());
    for item in headers {
        let header_rlp = convert_to_header_rlp(item).unwrap();
        total_difficulty = item.difficulty.as_u64().add(total_difficulty);
        let header_info = ETHHeaderInfo::new_builder()
            .header(hex::decode(header_rlp)?.into())
            .total_difficulty(total_difficulty.into())
            .hash(basic::Byte32::from_slice(item.hash.unwrap().as_bytes()).unwrap())
            .build();
        header_infos.push(header_info);
    }
    Ok(header_infos)
}

fn to_u64(data: &[u8]) -> u64 {
    let mut res = [0u8; 8];
    res.copy_from_slice(data);
    u64::from_le_bytes(res)
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ETHSPVProofJson {
    pub log_index: u64,
    pub log_entry_data: String,
    pub receipt_index: u64,
    pub receipt_data: String,
    pub header_data: String,
    pub proof: Vec<String>,
    pub token: H160,
    pub lock_amount: u128,
    pub bridge_fee: u128,
    pub recipient_lockscript: Vec<u8>,
    pub replay_resist_outpoint: Vec<u8>,
    pub sudt_extra_data: Vec<u8>,
    pub eth_address: H160,
    pub sender: H160,
}

impl TryFrom<ETHSPVProofJson> for witness::ETHSPVProof {
    type Error = anyhow::Error;
    fn try_from(proof: ETHSPVProofJson) -> Result<Self> {
        let mut proof_vec: Vec<basic::Bytes> = vec![];
        for i in 0..proof.proof.len() {
            // proof_vec.push(proof.proof[i].to_vec().into())
            proof_vec.push(hex::decode(&proof.proof[i]).unwrap().into())
        }
        Ok(witness::ETHSPVProof::new_builder()
            .log_index(proof.log_index.into())
            .receipt_index(proof.receipt_index.into())
            .receipt_data(hex::decode(clear_0x(&proof.receipt_data))?.into())
            .header_data(hex::decode(clear_0x(&proof.header_data))?.into())
            .proof(BytesVec::new_builder().set(proof_vec).build())
            .build())
    }
}

pub fn clear_0x(s: &str) -> &str {
    if &s[..2] == "0x" || &s[..2] == "0X" {
        &s[2..]
    } else {
        s
    }
}

#[derive(Clone)]
pub struct EthWitness {
    pub cell_dep_index_list: Vec<u8>,
    pub spv_proof: ETHSPVProofJson,
    pub compiled_merkle_proof: Vec<u8>,
}

impl EthWitness {
    pub fn as_bytes(&self) -> Bytes {
        let spv_proof: witness::ETHSPVProof = self
            .spv_proof
            .clone()
            .try_into()
            .expect("try into mint_xt_witness::ETHSPVProof success");
        let spv_proof = spv_proof.as_slice().to_vec();
        let witness_data = witness::MintTokenWitness::new_builder()
            .spv_proof(spv_proof.into())
            .cell_dep_index_list(self.cell_dep_index_list.clone().into())
            .merkle_proof(self.compiled_merkle_proof.clone().into())
            .build();
        let witness = WitnessArgs::new_builder()
            .lock(Some(witness_data.as_bytes()).pack())
            .build();
        witness.as_bytes()
    }
}
