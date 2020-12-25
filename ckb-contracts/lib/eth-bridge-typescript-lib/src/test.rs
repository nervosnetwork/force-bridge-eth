use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::Pack;
use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{self, Script, WitnessArgs},
};
use ckb_std::error::SysError;
use contracts_helper::data_loader::MockDataLoader;
use force_eth_types::config::{SUDT_CODE_HASH, SUDT_HASH_TYPE};
use force_eth_types::generated::basic::Byte32;
use force_eth_types::generated::eth_bridge_type_cell::{ETHBridgeTypeArgs, ETHBridgeTypeData};
use force_eth_types::generated::witness::MintTokenWitness;
use molecule::prelude::Byte;
use molecule::prelude::{Builder, Entity};

fn generate_manage_mode_correct_mock() -> MockDataLoader {
    let mut mock = MockDataLoader::new();

    let owner_script = Script::new_builder().args([1u8; 1].pack()).build();
    let data = ETHBridgeTypeData::new_builder()
        .owner_lock_script(owner_script.as_slice().to_vec().into())
        .build();

    mock.expect_load_cell_data()
        .times(2)
        .returning(move |index, _| {
            if index == 0 {
                Ok(data.clone().as_slice().to_vec())
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    let witness = MintTokenWitness::new_builder().mode(Byte::new(1u8)).build();
    let witness_args = WitnessArgs::new_builder()
        .lock(Some(witness.as_bytes()).pack())
        .build();
    mock.expect_load_witness_args()
        .times(1)
        .returning(move |_, _| Ok(witness_args.clone()));

    let script = Script::new_builder()
        .args(ETHBridgeTypeArgs::default().as_bytes().pack())
        .build();
    mock.expect_load_script()
        .times(1)
        .returning(move || Ok(script.clone()));

    mock
}

fn generate_mint_token_correct_mock() -> MockDataLoader {
    let correct_input_lock_hash = [100u8; 32];
    let correct_recipient_lock_hash = [101u8; 32];
    let correct_input_type_hash = [102u8; 32];
    let correct_owner_lockscript = [
        53u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 230, 131, 176, 65, 57, 52, 71, 104,
        52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131,
        31, 54, 96, 215, 0, 0, 0, 0, 0,
    ];

    let mut mock = MockDataLoader::new();

    let witness = MintTokenWitness::default();
    let witness_args = WitnessArgs::new_builder()
        .lock(Some(witness.as_bytes()).pack())
        .build();
    mock.expect_load_witness_args()
        .times(1)
        .returning(move |_, _| Ok(witness_args.clone()));

    let args = ETHBridgeTypeArgs::new_builder()
        .bridge_lock_hash(Byte32::new_unchecked(
            correct_input_lock_hash.to_vec().into(),
        ))
        .recipient_lock_hash(Byte32::new_unchecked(
            correct_recipient_lock_hash.to_vec().into(),
        ))
        .build();
    let script = Script::new_builder().args(args.as_bytes().pack()).build();
    mock.expect_load_script()
        .times(1)
        .returning(move || Ok(script.clone()));

    let data = ETHBridgeTypeData::new_builder()
        .owner_lock_script(correct_owner_lockscript.to_vec().into())
        .fee(10u128.into())
        .build();

    mock.expect_load_cell_data()
        .times(2)
        .returning(move |index, _| {
            if index == 0 {
                Ok(data.clone().as_slice().to_vec())
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    mock.expect_load_cell_lock_hash()
        .times(2)
        .returning(move |_, source| {
            if source == Source::Input {
                Ok(correct_input_lock_hash)
            } else {
                Ok(correct_recipient_lock_hash)
            }
        });

    mock.expect_load_cell_type_hash()
        .times(1)
        .returning(move |_, _| Ok(Some(correct_input_type_hash)));

    mock.expect_load_script_hash()
        .times(1)
        .returning(move || Ok(correct_input_type_hash));

    mock.expect_load_cell_lock()
        .times(1)
        .returning(move |_, _| Ok(Script::from_slice(&correct_owner_lockscript.clone()).unwrap()));

    mock.expect_load_cell_data()
        .times(1)
        .returning(|_, _| Ok(10u128.to_le_bytes().to_vec()));

    mock
}

#[test]
fn test_correct_manage_mode() {
    let mut mock = generate_manage_mode_correct_mock();

    let owner_script = Script::new_builder().args([1u8; 1].pack()).build();

    mock.expect_load_cell_lock()
        .times(1)
        .returning(move |_, _| Ok(owner_script.clone()));

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
#[should_panic(expected = "not authorized to unlock the cell")]
fn test_manage_mode_when_lock_script_not_exist_in_inputs() {
    let mut mock = generate_manage_mode_correct_mock();

    let invalid_owner_script = Script::new_builder().args([2u8; 1].pack()).build();

    mock.expect_load_cell_lock()
        .times(2)
        .returning(move |index, _| {
            if index == 0 {
                Ok(invalid_owner_script.clone())
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
fn test_mint_token() {
    let correct_input_lock_hash = [100u8; 32];
    let correct_sudt_script = Script::new_builder()
        .code_hash(packed::Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
        .hash_type(SUDT_HASH_TYPE.into())
        .args(Bytes::from(correct_input_lock_hash.to_vec()).pack())
        .build();

    let mut mock = generate_mint_token_correct_mock();

    mock.expect_load_cell_type()
        .times(4)
        .returning(move |index, _| {
            if index == 0 || index == 1 {
                Ok(Some(correct_sudt_script.clone()))
            } else if index == 2 {
                Ok(Some(Script::default()))
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
#[should_panic(expected = "mint more sudt than expected")]
fn test_mint_token_when_mint_more_sudt_than_expected() {
    let correct_input_lock_hash = [100u8; 32];
    let correct_sudt_script = Script::new_builder()
        .code_hash(packed::Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
        .hash_type(SUDT_HASH_TYPE.into())
        .args(Bytes::from(correct_input_lock_hash.to_vec()).pack())
        .build();
    let mut mock = generate_mint_token_correct_mock();

    mock.expect_load_cell_type()
        .times(3)
        .returning(move |_, _| Ok(Some(correct_sudt_script.clone())));

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}
