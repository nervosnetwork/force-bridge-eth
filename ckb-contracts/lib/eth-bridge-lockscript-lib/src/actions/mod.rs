use crate::adapter::Adapter;
use crate::debug;
use force_eth_types::generated::eth_header_cell::{EthCellDataReader, HeaderInfoReader};
use force_eth_types::generated::witness::{ETHSPVProofReader, MintTokenWitnessReader};

use eth_spv_lib::{eth_types::*, ethspv};

use molecule::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct ETHReceiptInfo {
    token_amount: [u8; 32],
    token_address: [u8; 32],
    ckb_recipient_address: [u8; 32],
    replay_resist_cell_id: [u8; 32],
}

pub fn verify_mint_token<T: Adapter>(data_loader: T, input_data: &[u8], output_data: &[u8]) -> i8 {
    verify_data(input_data, output_data);
    if !input_data.is_empty() && !data_loader.check_inputs_lock_hash(input_data) {
        panic!("verify signature fail")
    }

    verify_eth_light_client();
    let eth_receipt_info = verify_witness(data_loader);
    verify_eth_receipt_info(eth_receipt_info);
    0
}

pub fn verify_destroy_cell<T: Adapter>(_data_loader: T, _input_data: &[u8]) -> i8 {
    0
}

fn verify_data(input_data: &[u8], output_data: &[u8]) {
    if input_data != output_data {
        panic!("data changed")
    }
}

// fn verify_signature<T: Adapter>(data_loader: T, data: &[u8]) {
//     if !data_loader.check_inputs_lock_hash(data) {
//         panic!("verify signature fail")
//     }
// }

fn verify_eth_light_client() {
    todo!()
}

/// Verify eth witness data.
/// 1. Verify that the header of the user's cross-chain tx is on the main chain.
/// 2. Verify that the user's cross-chain transaction is legal and really exists (based spv proof).
/// 3. Get ETHReceiptInfo from spv proof.
///
fn verify_witness<T: Adapter>(data_loader: T) -> ETHReceiptInfo {
    let witness_args = data_loader
        .load_input_witness_args()
        .expect("load witness args error");
    if MintTokenWitnessReader::verify(&witness_args, false).is_err() {
        panic!("witness is invalid");
    }
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
    data_loader: T,
    proof: &[u8],
    cell_dep_index_list: &[u8],
) -> ETHReceiptInfo {
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
    data_loader: T,
    header: &BlockHeader,
    cell_dep_index_list: &[u8],
) {
    let dep_data = data_loader
        .load_cell_dep_data(cell_dep_index_list[0].into())
        .expect("load cell dep data failed");
    debug!("dep data is {:?}", &dep_data);

    if EthCellDataReader::verify(&dep_data, false).is_err() {
        panic!("eth cell data invalid");
    }

    let eth_cell_data_reader = EthCellDataReader::new_unchecked(&dep_data);
    debug!("eth_cell_data_reader: {:?}", eth_cell_data_reader);
    let tail_raw = eth_cell_data_reader
        .headers()
        .main()
        .get_unchecked(eth_cell_data_reader.headers().main().len() - 1)
        .raw_data();
    if HeaderInfoReader::verify(&tail_raw, false).is_err() {
        panic!("header info invalid");
    }
    let tail_info_reader = HeaderInfoReader::new_unchecked(tail_raw);
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
        .raw_data();
    let target_info_reader = HeaderInfoReader::new_unchecked(target_raw);
    debug!(
        "main chain hash: {:?}, witness header hash: {:?}",
        hex::encode(target_info_reader.hash().raw_data()),
        hex::encode(header.hash.expect("invalid hash").0.as_bytes())
    );
    if target_info_reader.hash().raw_data() != header.hash.expect("invalid hash").0.as_bytes() {
        panic!("header is not on mainchain, target not in eth data");
    }
}

fn get_eth_receipt_info(proof_reader: ETHSPVProofReader, header: BlockHeader) -> ETHReceiptInfo {
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

    let log_data = log_entry.data;
    let slices = slice_data(log_data.as_slice());
    debug!("log data slice: {:?}", slices);

    let token_amount = slices[0];
    let token_address = slices[1];
    let ckb_recipient_address = slices[2];
    let replay_resist_cell_id = slices[3];

    let eth_receipt_info = ETHReceiptInfo {
        token_amount,
        token_address,
        ckb_recipient_address,
        replay_resist_cell_id,
    };
    debug!("log data eth_receipt_info: {:?}", eth_receipt_info);

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
fn verify_eth_receipt_info(_eth_receipt_info: ETHReceiptInfo) {}
