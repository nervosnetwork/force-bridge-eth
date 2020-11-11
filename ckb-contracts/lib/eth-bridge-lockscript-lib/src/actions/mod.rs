use crate::adapter::Adapter;
use crate::debug;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32, Script},
    prelude::Pack,
};
use ckb_std::high_level::QueryIter;
use eth_spv_lib::{eth_types::*, ethspv};
use force_eth_types::generated::eth_header_cell::{
    ETHHeaderCellData, ETHHeaderCellDataReader, ETHHeaderInfoReader,
};
use force_eth_types::generated::witness::{ETHSPVProofReader, MintTokenWitnessReader};
use force_eth_types::{
    config::SUDT_CODE_HASH, eth_lock_event::ETHLockEvent, util::eth_to_ckb_amount,
};
use molecule::prelude::*;
use std::convert::TryInto;

pub enum Mode {
    Owner,
    Mint,
}

/// if the first 32 bytes of data match any of input lockscript hash,
/// it is owner mode, otherwise it is mint mode.
pub fn check_mode<T: Adapter>(data_loader: &T) -> Mode {
    let input_data = data_loader.load_input_data();
    if input_data.len() >= 32 && data_loader.lock_hash_exists_in_inputs(&input_data[..32]) {
        Mode::Owner
    } else {
        Mode::Mint
    }
}

/// In owner mode, mint associated sudt is forbidden.
/// Owners can do options like destroy the cell or supply capacity for it,
/// which means put an identical cell in output with higher capacity.
pub fn verify_owner_mode<T: Adapter>(data_loader: &T) {
    let udt_script = data_loader.get_associated_udt_script();
    if data_loader.typescript_exists_in_outputs(udt_script.as_slice()) {
        panic!("mint sudt is forbidden in owner mode");
    }
}

pub fn verify_mint_token<T: Adapter>(data_loader: &T) {
    verify_eth_light_client();
    let eth_receipt_info = verify_witness(data_loader);
    verify_eth_receipt_info(data_loader, eth_receipt_info);
}

pub fn verify_destroy_cell<T: Adapter>(_data_loader: &T, _input_data: &[u8]) -> i8 {
    0
}

fn verify_data(input_data: &[u8], output_data: &[u8]) {
    if input_data != output_data {
        panic!("data changed")
    }
}

// fn verify_signature<T: Adapter>(data_loader: &T, data: &[u8]) {
//     if !data_loader.check_inputs_lock_hash(data) {
//         panic!("verify signature fail")
//     }
// }

fn verify_eth_light_client() {
    // todo!()
}

/// Verify eth witness data.
/// 1. Verify that the header of the user's cross-chain tx is on the main chain.
/// 2. Verify that the user's cross-chain transaction is legal and really exists (based spv proof).
/// 3. Get ETHLockEvent from spv proof.
///
fn verify_witness<T: Adapter>(data_loader: &T) -> ETHLockEvent {
    let witness_args = data_loader
        .load_input_witness_args()
        .expect("load witness args error");
    MintTokenWitnessReader::verify(&witness_args, false).expect("witness is invalid");
    let witness = MintTokenWitnessReader::new_unchecked(&witness_args);
    debug!("witness: {:?}", witness);
    let proof = witness.spv_proof().raw_data();
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();

    verify_eth_spv_proof(data_loader, proof, cell_dep_index_list)
}

/// Verify eth witness data.
/// 1. Verify that the header of the user's cross-chain tx is on the main chain.
/// 2. Verify that the user's cross-chain transaction is legal and really exists (based spv proof).
/// @param data is used to get the real lock address.
/// @param proof is the spv proof data for cross-chain tx.
/// @param cell_dep_index_list is used to get the headers oracle information to verify the cross-chain tx is really exists on the main chain.
///
fn verify_eth_spv_proof<T: Adapter>(
    data_loader: &T,
    proof: &[u8],
    cell_dep_index_list: &[u8],
) -> ETHLockEvent {
    if ETHSPVProofReader::verify(proof, false).is_err() {
        panic!("eth spv proof is invalid")
    }
    let proof_reader = ETHSPVProofReader::new_unchecked(proof);
    let header_data = proof_reader.header_data().raw_data().to_vec();
    let header: BlockHeader = rlp::decode(header_data.as_slice()).expect("invalid header data");
    debug!("the spv proof header data: {:?}", header);

    //verify the header is on main chain.
    verify_eth_header_on_main_chain(data_loader, &header, cell_dep_index_list);

    get_eth_receipt_info(proof_reader, header)
}

fn verify_eth_header_on_main_chain<T: Adapter>(
    data_loader: &T,
    header: &BlockHeader,
    cell_dep_index_list: &[u8],
) {
    let dep_data = data_loader
        .load_cell_dep_data(cell_dep_index_list[0].into())
        .expect("load cell dep data failed");
    debug!("dep data is {:?}", &dep_data);

    if ETHHeaderCellDataReader::verify(&dep_data, false).is_err() {
        panic!("eth cell data invalid");
    }

    let eth_cell_data_reader = ETHHeaderCellDataReader::new_unchecked(&dep_data);
    debug!("eth_cell_data_reader: {:?}", eth_cell_data_reader);
    let tail_raw = eth_cell_data_reader
        .headers()
        .main()
        .get_unchecked(eth_cell_data_reader.headers().main().len() - 1)
        .raw_data();
    if ETHHeaderInfoReader::verify(&tail_raw, false).is_err() {
        panic!("header info invalid");
    }
    let tail_info_reader = ETHHeaderInfoReader::new_unchecked(&tail_raw);
    let tail_info_raw = tail_info_reader.header().raw_data();
    let tail: BlockHeader =
        rlp::decode(tail_info_raw.to_vec().as_slice()).expect("invalid tail info.");
    if header.number > tail.number {
        panic!("header is not on mainchain, header number too big");
    }
    let offset = (tail.number - header.number) as usize;
    if offset > eth_cell_data_reader.headers().main().len() - 1 {
        panic!("header is not on mainchain, header number is too small");
    }
    let target_raw = eth_cell_data_reader
        .headers()
        .main()
        .get_unchecked(eth_cell_data_reader.headers().main().len() - 1 - offset)
        .raw_data()
        .as_ref();
    let target_info_reader = ETHHeaderInfoReader::new_unchecked(target_raw);
    debug!(
        "main chain hash: {:?}, witness header hash: {:?}",
        hex::encode(target_info_reader.hash().raw_data()),
        hex::encode(header.hash.expect("invalid hash").0.as_bytes())
    );
    if target_info_reader.hash().raw_data() != header.hash.expect("invalid hash").0.as_bytes() {
        panic!("header is not on mainchain, target not in eth data");
    }
}

fn get_eth_receipt_info(proof_reader: ETHSPVProofReader, header: BlockHeader) -> ETHLockEvent {
    let mut log_index = [0u8; 8];
    log_index.copy_from_slice(proof_reader.log_index().raw_data());
    debug!("log_index is {:?}", &log_index);

    let log_entry_data = proof_reader.log_entry_data().raw_data().to_vec();
    debug!(
        "log_entry_data is {:?}",
        hex::encode(&log_entry_data.as_slice())
    );

    let receipt_data = proof_reader.receipt_data().raw_data().to_vec();
    debug!(
        "receipt_data is {:?}",
        hex::encode(&receipt_data.as_slice())
    );

    let mut receipt_index = [0u8; 8];
    receipt_index.copy_from_slice(proof_reader.receipt_index().raw_data());
    debug!("receipt_index is {:?}", &receipt_index);

    let mut proof = vec![];
    for i in 0..proof_reader.proof().len() {
        proof.push(proof_reader.proof().get_unchecked(i).raw_data().to_vec());
    }
    debug!("proof: {:?}", hex::encode(proof[0].clone()));

    let log_entry: LogEntry =
        rlp::decode(log_entry_data.as_slice()).expect("rlp decode log_entry failed");
    debug!("log_entry is {:?}", &log_entry);

    let receipt: Receipt = rlp::decode(receipt_data.as_slice()).expect("rlp decode receipt failed");
    debug!("receipt_data is {:?}", &receipt);

    if !ethspv::verify_log_entry(
        u64::from_le_bytes(log_index),
        log_entry_data,
        u64::from_le_bytes(receipt_index),
        receipt_data,
        header.receipts_root,
        proof,
    ) {
        panic!("wrong merkle proof");
    }

    let log_data = log_entry.data;
    let eth_receipt_info = ETHLockEvent::parse_from_event_data(&log_data);
    debug!("log data eth_receipt_info: {:?}", eth_receipt_info);
    eth_receipt_info
}

/// Converts a vector of bytes with len equal n * 32, to a vector of slices.
fn slice_data(data: &[u8]) -> Vec<[u8; 32]> {
    if data.len() % 32 != 0 {
        panic!("log data encoding error");
    }

    let times = data.len() / 32;
    let mut result = Vec::with_capacity(times);
    for i in 0..times {
        let mut slice = [0u8; 32];
        let offset = 32 * i;
        slice.copy_from_slice(&data[offset..offset + 32]);
        result.push(slice);
    }
    result
}

/// Verify eth receipt info.
/// 1. Verify ckb_recipient_address get a number of token_amount cToken.
/// 2. Verify token_address equals to args.token_address.
/// 3. Verify replay_resist_cell_id exists in inputs.
fn verify_eth_receipt_info<T: Adapter>(data_loader: &T, eth_receipt_info: ETHLockEvent) {
    if !data_loader.outpoint_exists_in_inputs(eth_receipt_info.replay_resist_outpoint.as_ref()) {
        panic!("replay_resist_cell_id not exists in inputs");
    }
    let udt_typescript = data_loader.get_associated_udt_script();
    let udt_script_slice = udt_typescript.as_slice();
    let expected_mint_amount: u128 =
        eth_to_ckb_amount(eth_receipt_info.locked_amount).expect("locked amount overflow");
    let bridge_fee: u128 =
        eth_to_ckb_amount(eth_receipt_info.bridge_fee).expect("bridge fee overflow");
    let mut mint_amount = 0u128;
    let mut recipient_amount = 0u128;
    for (output_type, output_lock, output_data) in QueryIter::new(
        |index, source| data_loader.load_cell_type_lock_data(index, source),
        Source::Output,
    )
    .into_iter()
    {
        if output_type.is_some() && udt_script_slice == output_type.unwrap().as_slice() {
            let mut amount = [0u8; 16];
            amount.copy_from_slice(&output_data[..16]);
            let amount = u128::from_le_bytes(amount);
            mint_amount += amount;
            if output_lock.as_slice() == eth_receipt_info.recipient_lockscript.as_slice() {
                if recipient_amount != 0 {
                    panic!("you can only mint one sudt cell for recipient");
                }
                assert_eq!(
                    &output_data[16..],
                    eth_receipt_info.sudt_extra_data.as_slice(),
                    "recipient sudt cell extra data not match"
                );
                recipient_amount += amount;
            }
        }
    }
    assert_eq!(
        mint_amount, expected_mint_amount,
        "mint token amount not equal to expected"
    );
    assert_eq!(
        recipient_amount,
        expected_mint_amount - bridge_fee,
        "recipient amount not equal to expected(mint_amount - bridge_fee)"
    );
}
