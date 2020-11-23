use crate::adapter::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use force_eth_types::generated::eth_bridge_type_cell::{ETHBridgeTypeArgs, ETHBridgeTypeData};
use molecule::prelude::*;

pub fn verify_manage_mode<T: Adapter>(data_loader: &T, owner: &[u8]) {
    if !data_loader.lock_script_exists_in_inputs(owner) {
        panic!("not authorized to unlock the cell");
    }
}

pub fn verify_mint_token<T: Adapter>(
    data_loader: &T,
    script_args: &ETHBridgeTypeArgs,
    data: &ETHBridgeTypeData,
) {
    // verify first input cell is bridge cell
    assert_eq!(
        &data_loader.load_cell_lock_hash(0, Source::Input).unwrap(),
        script_args.bridge_lock_hash().as_slice(),
    );
    assert_eq!(
        data_loader
            .load_cell_type_hash(0, Source::Input)
            .unwrap()
            .unwrap(),
        data_loader.load_script_hash(),
    );
    let udt_typescript =
        data_loader.get_associated_udt_script(script_args.bridge_lock_hash().as_slice());
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
    let mut index = 1;
    // verify 2nd output is fee sudt cell
    if data.fee().as_slice().iter().any(|&b| b != 0) {
        let second_output_typescript = data_loader
            .load_cell_type(index, Source::Output)
            .unwrap()
            .unwrap();
        assert_eq!(sudt_typescript_slice, second_output_typescript.as_slice());
        let second_output_lock_script = data_loader
            .load_cell_lock_script(index, Source::Output)
            .unwrap();
        assert_eq!(
            second_output_lock_script.as_bytes(),
            data.owner_lock_script().raw_data()
        );
        let second_output_data = data_loader.load_cell_data(index, Source::Output).unwrap();
        assert_eq!(&second_output_data[..16], data.fee().as_slice());
        index += 1;
    }
    // verify there are no other sudt cell
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
            Ok(None) => {}
        }
        index += 1;
    }
}
