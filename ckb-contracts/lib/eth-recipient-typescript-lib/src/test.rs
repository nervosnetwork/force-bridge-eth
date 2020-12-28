use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::Pack;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{self, Script},
};
use ckb_std::error::SysError;
use contracts_helper::data_loader::MockDataLoader;
use core::convert::TryFrom;
use force_eth_types::config::{SUDT_CODE_HASH, SUDT_HASH_TYPE};
use force_eth_types::eth_recipient_cell::{ETHAddress, ETHRecipientDataView};
use molecule::prelude::{Builder, Entity};

fn generate_correct_mock() -> MockDataLoader {
    let mut mock = MockDataLoader::new();

    let correct_input_sudt_amount: u128 = 100;
    let correct_output_sudt_amount: u128 = 90;
    let correct_token_amount: u128 = 10;
    let correct_fee: u128 = 1;

    let data = ETHRecipientDataView {
        eth_recipient_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
        eth_token_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
        eth_lock_contract_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
        eth_bridge_lock_hash: [1u8; 32],
        light_client_typescript_hash: [1u8; 32],
        token_amount: correct_token_amount,
        fee: correct_fee,
    };

    mock.expect_load_cell_data()
        .times(4)
        .returning(move |index, source| {
            if source == Source::GroupOutput {
                if index == 0 {
                    Ok(data.as_molecule_data().unwrap().to_vec())
                } else {
                    Err(SysError::IndexOutOfBound)
                }
            } else if source == Source::Input {
                Ok(correct_input_sudt_amount.to_le_bytes().to_vec())
            } else {
                Ok(correct_output_sudt_amount.to_le_bytes().to_vec())
            }
        });

    let correct_bridge_lock_hash = [
        33u8, 128, 167, 78, 171, 136, 228, 5, 173, 35, 191, 141, 144, 148, 186, 90, 153, 104, 73,
        131, 30, 154, 184, 165, 113, 41, 201, 242, 100, 41, 140, 197,
    ];
    let correct_sudt_script = Script::new_builder()
        .code_hash(packed::Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
        .hash_type(SUDT_HASH_TYPE.into())
        .args(Bytes::from(correct_bridge_lock_hash.to_vec()).pack())
        .build();

    mock.expect_load_cell_type()
        .times(4)
        .returning(move |index, _| {
            if index == 0 {
                Ok(Some(correct_sudt_script.clone()))
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    mock
}

#[test]
fn test_burn_token_correct() {
    let mock = generate_correct_mock();

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}
