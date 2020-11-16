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
use force_eth_types::generated::eth_bridge_type_cell::ETHBridgeTypeArgs;
use force_eth_types::generated::eth_header_cell::{
    ETHHeaderCellData, ETHHeaderCellDataReader, ETHHeaderInfoReader,
};
use force_eth_types::generated::witness::{ETHSPVProofReader, MintTokenWitnessReader};
use force_eth_types::{
    config::SUDT_CODE_HASH, eth_lock_event::ETHLockEvent, util::eth_to_ckb_amount,
};
use molecule::prelude::*;
use std::convert::TryInto;
use ckb_std::error::SysError;

pub fn verify_manage_mode<T: Adapter>(data_loader: &T, owner: &[u8]) {
    if !data_loader.lock_hash_exists_in_inputs(owner) {
        panic!("not authorized to unlock the cell");
    }
}

pub fn verify_mint_token<T: Adapter>(
    data_loader: &T,
    witness: &MintTokenWitnessReader,
    script_args: &ETHBridgeTypeArgs,
) {
    // let eth_receipt_info = parse_event_from_witness(witness);
    // verify first input cell is bridge cell
    assert_eq!(
        &data_loader.load_cell_lock_hash(0, Source::Input).unwrap(),
        script_args.bridge_lock_hash().as_slice(),
    );
    assert_eq!(
        data_loader.load_cell_type_hash(0, Source::Input).unwrap().unwrap(),
        data_loader.load_script_hash(),
    );

    let udt_typescript = data_loader.get_associated_udt_script();
    let sudt_typescript_slice = udt_typescript.as_slice();
    // verify 1st output is recipient sudt cell
    let first_output_typescript = data_loader
        .load_cell_type(0, Source::Output)
        .unwrap()
        .unwrap();
    assert_eq!(sudt_typescript_slice, first_output_typescript.as_slice());
    let first_output_lock_hash = data_loader.load_cell_lock_hash(0, Source::Output).unwrap();
    assert_eq!(
        &first_output_lock_hash,
        script_args.recipient_lock_hash().as_slice()
    );
    // verify 2nd output is fee sudt cell
    let second_output_typescript = data_loader
        .load_cell_type(1, Source::Output)
        .unwrap()
        .unwrap();
    assert_eq!(sudt_typescript_slice, second_output_typescript.as_slice());
    let second_output_lock_hash = data_loader.load_cell_lock_hash(1, Source::Output).unwrap();
    assert_eq!(
        &second_output_lock_hash,
        script_args.owner_lock_hash().as_slice()
    );
    let second_output_data = data_loader.load_cell_data(1, Source::Output).unwrap();
    assert_eq!(&second_output_data[..16], script_args.fee().as_slice());
    // verify there are no other sudt cell
    let mut index = 2;
    loop {
        let typescript_res = data_loader.load_cell_type(index, Source::Output);
        match typescript_res {
            Err(SysError::IndexOutOfBound) => break,
            Err(_err) => panic!("iter output return an error"),
            Ok(Some(cell_type)) => {
                if cell_type.as_slice() == sudt_typescript_slice {
                    panic!("mint more sudt than expected");
                }
            }
            Ok(None) => {},
        }
        index += 1;
    }
}

fn parse_event_from_witness(witness: &MintTokenWitnessReader) -> ETHLockEvent {
    let proof = witness.spv_proof().raw_data();
    let proof_reader = ETHSPVProofReader::new_unchecked(proof);
    let log_entry_data = proof_reader.log_entry_data().raw_data().to_vec();
    let log_entry: LogEntry =
        rlp::decode(log_entry_data.as_slice()).expect("rlp decode log_entry failed");
    let log_data = log_entry.data;
    let eth_receipt_info = ETHLockEvent::parse_from_event_data(&log_data);
    eth_receipt_info
}
