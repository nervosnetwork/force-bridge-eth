use anyhow::{anyhow, Result};
use ckb_hash::{blake2b_256, new_blake2b};
use ckb_jsonrpc_types as rpc_types;
use ckb_sdk::constants::{MULTISIG_TYPE_HASH, SECP_SIGNATURE_SIZE, SIGHASH_TYPE_HASH};
use ckb_sdk::{Address, AddressPayload, HttpRpcClient, MultisigConfig, SECP256K1};
use ckb_types::bytes::{Bytes, BytesMut};
use ckb_types::core::{ScriptHashType, TransactionBuilder, TransactionView};
use ckb_types::packed::{
    Byte32, CellOutput, OutPoint, Script, ScriptReader, Transaction, WitnessArgs,
};
use ckb_types::prelude::*;
use ckb_types::{h256, packed, H160, H256};
use force_eth_types::generated::eth_header_cell::ETHHeaderCellMerkleDataReader;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::str::FromStr;

#[derive(Clone)]
pub struct TxHelper {
    pub transaction: TransactionView,
    multisig_configs: HashMap<H160, MultisigConfig>,
    signatures: HashMap<Bytes, HashSet<Bytes>>,
}

impl Default for TxHelper {
    fn default() -> TxHelper {
        TxHelper {
            transaction: TransactionBuilder::default().build(),
            multisig_configs: HashMap::default(),
            signatures: HashMap::default(),
        }
    }
}

impl TxHelper {
    pub fn new(transaction: TransactionView) -> TxHelper {
        TxHelper {
            transaction,
            multisig_configs: HashMap::default(),
            signatures: HashMap::default(),
        }
    }

    pub fn add_multisig_config(&mut self, config: MultisigConfig) {
        self.multisig_configs.insert(config.hash160(), config);
    }

    #[allow(clippy::mutable_key_type)]
    pub fn sign_inputs<C>(
        self,
        mut signer: SignerFn,
        get_live_cell: C,
        skip_check: bool,
    ) -> Result<HashMap<Bytes, Bytes>, String>
    where
        C: FnMut(OutPoint, bool) -> Result<CellOutput, String>,
    {
        let all_sighash_lock_args = self
            .multisig_configs
            .iter()
            .map(|(hash160, config)| (hash160.clone(), config.sighash_lock_args()))
            .collect::<HashMap<_, _>>();

        let witnesses = self.init_witnesses();
        let input_size = self.transaction.inputs().len();
        let mut signatures: HashMap<Bytes, Bytes> = Default::default();
        for ((code_hash, lock_arg), idxs) in
            self.input_group(get_live_cell, skip_check)?.into_iter()
        {
            if code_hash != SIGHASH_TYPE_HASH.pack() && code_hash != MULTISIG_TYPE_HASH.pack() {
                continue;
            }

            let multisig_hash160 = H160::from_slice(&lock_arg[..20]).unwrap();
            let lock_args = if code_hash == MULTISIG_TYPE_HASH.pack() {
                all_sighash_lock_args
                    .get(&multisig_hash160)
                    .unwrap()
                    .clone()
            } else {
                let mut lock_args = HashSet::default();
                lock_args.insert(H160::from_slice(lock_arg.as_ref()).unwrap());
                lock_args
            };
            if signer(&lock_args, &h256!("0x0"), &Transaction::default().into())?.is_some() {
                let signature = build_signature(
                    &self.transaction,
                    input_size,
                    &idxs,
                    &witnesses,
                    self.multisig_configs.get(&multisig_hash160),
                    |message: &H256, tx: &rpc_types::Transaction| {
                        signer(&lock_args, message, tx).map(|sig| sig.unwrap())
                    },
                )?;
                signatures.insert(lock_arg, signature);
            }
        }
        Ok(signatures)
    }

    #[allow(clippy::mutable_key_type)]
    pub fn input_group<F: FnMut(OutPoint, bool) -> Result<CellOutput, String>>(
        &self,
        mut get_live_cell: F,
        skip_check: bool,
    ) -> Result<HashMap<(Byte32, Bytes), Vec<usize>>, String> {
        let mut input_group: HashMap<(Byte32, Bytes), Vec<usize>> = HashMap::default();
        for (idx, input) in self.transaction.inputs().into_iter().enumerate() {
            let lock = get_live_cell(input.previous_output(), false)?.lock();
            check_lock_script(&lock, skip_check)
                .map_err(|err| format!("Input(no.{}) {}", idx + 1, err))?;

            let lock_arg = lock.args().raw_data();
            let code_hash = lock.code_hash();
            if code_hash == MULTISIG_TYPE_HASH.pack() {
                let hash160 = H160::from_slice(&lock_arg[..20]).unwrap();
                if !self.multisig_configs.contains_key(&hash160) {
                    return Err(format!(
                        "No mutisig config found for input(no.{}) lock_arg prefix: {:#x}",
                        idx + 1,
                        hash160,
                    ));
                }
            }
            input_group
                .entry((code_hash, lock_arg))
                .or_default()
                .push(idx);
        }
        Ok(input_group)
    }

    pub fn init_witnesses(&self) -> Vec<packed::Bytes> {
        let mut witnesses: Vec<packed::Bytes> = self.transaction.witnesses().into_iter().collect();
        while witnesses.len() < self.transaction.inputs().len() {
            witnesses.push(Bytes::new().pack());
        }
        witnesses
    }
}

pub fn generate_from_lockscript(from_privkey: SecretKey) -> Result<Script> {
    let from_public_key = secp256k1::PublicKey::from_secret_key(&SECP256K1, &from_privkey);
    let address_payload = AddressPayload::from_pubkey(&from_public_key);
    Ok(Script::from(&address_payload))
}

pub fn get_privkey_signer(privkey: SecretKey) -> SignerFn {
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let lock_arg = H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20])
        .expect("Generate hash(H160) from pubkey failed");
    Box::new(
        move |lock_args: &HashSet<H160>, message: &H256, _tx: &rpc_types::Transaction| {
            if lock_args.contains(&lock_arg) {
                if message == &h256!("0x0") {
                    Ok(Some([0u8; 65]))
                } else {
                    let message = secp256k1::Message::from_slice(message.as_bytes())
                        .expect("Convert to secp256k1 message failed");
                    let signature = SECP256K1.sign_recoverable(&message, &privkey);
                    Ok(Some(serialize_signature(&signature)))
                }
            } else {
                Ok(None)
            }
        },
    )
}

pub fn build_signature<
    S: FnMut(&H256, &rpc_types::Transaction) -> Result<[u8; SECP_SIGNATURE_SIZE], String>,
>(
    tx: &TransactionView,
    input_size: usize,
    input_group_idxs: &[usize],
    witnesses: &[packed::Bytes],
    multisig_config_opt: Option<&MultisigConfig>,
    mut signer: S,
) -> Result<Bytes, String> {
    let init_witness_idx = input_group_idxs[0];
    let init_witness = if witnesses[init_witness_idx].raw_data().is_empty() {
        WitnessArgs::default()
    } else {
        WitnessArgs::from_slice(witnesses[init_witness_idx].raw_data().as_ref())
            .map_err(|err| err.to_string())?
    };

    let init_witness = if let Some(multisig_config) = multisig_config_opt {
        let lock_without_sig = {
            let sig_len = (multisig_config.threshold() as usize) * SECP_SIGNATURE_SIZE;
            let mut data = BytesMut::from(&multisig_config.to_witness_data()[..]);
            data.extend_from_slice(vec![0u8; sig_len].as_slice());
            data.freeze()
        };
        init_witness
            .as_builder()
            .lock(Some(lock_without_sig).pack())
            .build()
    } else {
        init_witness
            .as_builder()
            .lock(Some(Bytes::from(vec![0u8; SECP_SIGNATURE_SIZE])).pack())
            .build()
    };

    let mut blake2b = new_blake2b();
    blake2b.update(tx.hash().as_slice());
    blake2b.update(&(init_witness.as_bytes().len() as u64).to_le_bytes());
    blake2b.update(&init_witness.as_bytes());
    for idx in input_group_idxs.iter().skip(1).cloned() {
        let other_witness: &packed::Bytes = &witnesses[idx];
        blake2b.update(&(other_witness.len() as u64).to_le_bytes());
        blake2b.update(&other_witness.raw_data());
    }
    for outter_witness in &witnesses[input_size..witnesses.len()] {
        blake2b.update(&(outter_witness.len() as u64).to_le_bytes());
        blake2b.update(&outter_witness.raw_data());
    }
    let mut message = [0u8; 32];
    blake2b.finalize(&mut message);
    let message = H256::from(message);
    signer(&message, &tx.data().into()).map(|data| Bytes::from(data.to_vec()))
}

pub fn serialize_signature(signature: &secp256k1::recovery::RecoverableSignature) -> [u8; 65] {
    let (recov_id, data) = signature.serialize_compact();
    let mut signature_bytes = [0u8; 65];
    signature_bytes[0..64].copy_from_slice(&data[0..64]);
    signature_bytes[64] = recov_id.to_i32() as u8;
    signature_bytes
}

pub fn check_lock_script(lock: &Script, skip_check: bool) -> Result<(), String> {
    #[derive(Eq, PartialEq)]
    enum CodeHashCategory {
        Sighash,
        Multisig,
        Other,
    }

    let code_hash: H256 = lock.code_hash().unpack();
    let hash_type: ScriptHashType = lock.hash_type().try_into().expect("hash_type");
    let lock_args = lock.args().raw_data();

    let code_hash_category = if code_hash == SIGHASH_TYPE_HASH {
        CodeHashCategory::Sighash
    } else if code_hash == MULTISIG_TYPE_HASH {
        CodeHashCategory::Multisig
    } else {
        CodeHashCategory::Other
    };
    let hash_type_str = if hash_type == ScriptHashType::Type {
        "type"
    } else {
        "data"
    };

    match (code_hash_category, hash_type, lock_args.len()) {
        (CodeHashCategory::Sighash, ScriptHashType::Type, 20) => Ok(()),
        (CodeHashCategory::Multisig, ScriptHashType::Type, 20) => Ok(()),
        (CodeHashCategory::Multisig, ScriptHashType::Type, 28) => Ok(()),
        (CodeHashCategory::Sighash, _, _) => Err(format!(
            "Invalid sighash lock script, hash_type: {}, args.length: {}",
            hash_type_str,
            lock_args.len()
        )),
        (CodeHashCategory::Multisig, _, _) => Err(format!(
            "Invalid multisig lock script, hash_type: {}, args.length: {}",
            hash_type_str,
            lock_args.len()
        )),
        (CodeHashCategory::Other, _, _) if skip_check => Ok(()),
        (CodeHashCategory::Other, _, _) => Err(format!(
            "invalid lock script code_hash: {:#x}, hash_type: {}, args.length: {}",
            code_hash,
            hash_type_str,
            lock_args.len(),
        )),
    }
}

#[allow(clippy::mutable_key_type)]
pub fn get_live_cell_with_cache(
    cache: &mut HashMap<(OutPoint, bool), (CellOutput, Bytes)>,
    client: &mut HttpRpcClient,
    out_point: OutPoint,
    with_data: bool,
) -> Result<(CellOutput, Bytes), String> {
    if let Some(output) = cache.get(&(out_point.clone(), with_data)).cloned() {
        Ok(output)
    } else {
        let output = get_live_cell(client, out_point.clone(), with_data)?;
        cache.insert((out_point, with_data), output.clone());
        Ok(output)
    }
}

pub fn get_live_cell(
    client: &mut HttpRpcClient,
    out_point: OutPoint,
    with_data: bool,
) -> Result<(CellOutput, Bytes), String> {
    let cell = client.get_live_cell(out_point.clone(), with_data)?;
    if cell.status != "live" {
        return Err(format!(
            "Invalid cell status: {}, out_point: {}",
            cell.status, out_point
        ));
    }
    let cell_status = cell.status.clone();
    cell.cell
        .map(|cell| {
            (
                cell.output.into(),
                cell.data
                    .map(|data| data.content.into_bytes())
                    .unwrap_or_default(),
            )
        })
        .ok_or_else(|| {
            format!(
                "Invalid input cell, status: {}, out_point: {}",
                cell_status, out_point
            )
        })
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

pub type SignerFn = Box<
    dyn FnMut(&HashSet<H160>, &H256, &rpc_types::Transaction) -> Result<Option<[u8; 65]>, String>,
>;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct MultisigConf {
    pub addresses: Vec<String>,
    pub require_first_n: u8,
    pub threshold: u8,
}

pub fn to_multisig_congif(conf: &MultisigConf) -> Result<MultisigConfig> {
    let mut addresses = vec![];
    for item in conf.addresses.clone() {
        let address = Address::from_str(&item).unwrap();
        addresses.push(address);
    }
    let sighash_addresses = addresses
        .into_iter()
        .map(|address| address.payload().clone())
        .collect::<Vec<_>>();

    let multisig_config =
        MultisigConfig::new_with(sighash_addresses, conf.require_first_n, conf.threshold)
            .map_err(|err| anyhow!(err))?;
    Ok(multisig_config)
}

pub fn parse_cell(cell: &str) -> Result<Script> {
    let cell_bytes =
        hex::decode(cell).map_err(|e| anyhow!("cell shoule be hex format, err: {}", e))?;
    ScriptReader::verify(&cell_bytes, false).map_err(|e| anyhow!("cell decoding err: {}", e))?;
    let cell_typescript = Script::new_unchecked(cell_bytes.into());
    Ok(cell_typescript)
}
