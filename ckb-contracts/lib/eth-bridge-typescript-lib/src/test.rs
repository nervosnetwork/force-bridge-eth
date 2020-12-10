use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::Script;
use ckb_std::ckb_types::prelude::Pack;
use ckb_std::error::SysError;
use force_eth_types::generated::basic::Byte32;
use force_eth_types::generated::eth_bridge_type_cell::{ETHBridgeTypeArgs, ETHBridgeTypeData};
use force_eth_types::generated::witness::MintTokenWitness;
use molecule::prelude::Byte;
use molecule::prelude::{Builder, Entity};

#[test]
fn test_manage_mode() {
    let mut mock = MockAdapter::new();

    let witness = MintTokenWitness::new_builder().mode(Byte::new(1u8)).build();

    mock.expect_load_input_witness_args()
        .times(1)
        .returning(move || Ok(witness.as_bytes()));

    #[allow(clippy::redundant_closure)]
    mock.expect_load_script_args()
        .times(1)
        .returning(|| ETHBridgeTypeArgs::default());

    mock.expect_load_data()
        .times(1)
        .returning(|| Some(ETHBridgeTypeData::default()));

    mock.expect_lock_script_exists_in_inputs()
        .times(1)
        .returning(|_| true);

    _verify(mock);
}

#[test]
#[should_panic(expected = "not authorized to unlock the cell")]
fn test_manage_mode_when_lock_script_not_exist_in_inputs() {
    let mut mock = MockAdapter::new();

    let witness = MintTokenWitness::new_builder().mode(Byte::new(1u8)).build();

    mock.expect_load_input_witness_args()
        .times(1)
        .returning(move || Ok(witness.as_bytes()));

    #[allow(clippy::redundant_closure)]
    mock.expect_load_script_args()
        .times(1)
        .returning(move || ETHBridgeTypeArgs::default());

    mock.expect_load_data()
        .times(1)
        .returning(move || Some(ETHBridgeTypeData::default()));

    mock.expect_lock_script_exists_in_inputs()
        .times(1)
        .returning(|_| false);

    _verify(mock);
}

fn generate_mint_token_params() -> MockAdapter {
    let correct_input_lock_hash = [100u8; 32];
    let correct_recipient_lock_hash = [101u8; 32];
    let correct_input_type_hash = [102u8; 32];
    let correct_sudt_code_hash = [103u8; 32];
    let correct_owner_lockscript = [
        53u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 230, 131, 176, 65, 57, 52, 71, 104,
        52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131,
        31, 54, 96, 215, 0, 0, 0, 0, 0,
    ];

    let mut mock = MockAdapter::new();

    mock.expect_load_input_witness_args()
        .times(1)
        .returning(move || Ok(MintTokenWitness::default().as_bytes()));

    mock.expect_load_script_args().times(1).returning(move || {
        ETHBridgeTypeArgs::new_builder()
            .bridge_lock_hash(Byte32::new_unchecked(
                correct_input_lock_hash.to_vec().into(),
            ))
            .recipient_lock_hash(Byte32::new_unchecked(
                correct_recipient_lock_hash.to_vec().into(),
            ))
            .build()
    });

    mock.expect_load_data().times(1).returning(move || {
        Some(
            ETHBridgeTypeData::new_builder()
                .owner_lock_script(correct_owner_lockscript.to_vec().into())
                .fee(10u128.into())
                .build(),
        )
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
        .returning(move || correct_input_type_hash);

    mock.expect_get_associated_udt_script()
        .times(1)
        .returning(move |_| {
            Script::new_builder()
                .code_hash(correct_sudt_code_hash.clone().pack())
                .build()
        });

    mock.expect_load_cell_lock_script()
        .times(1)
        .returning(move |_, _| Ok(Script::from_slice(&correct_owner_lockscript.clone()).unwrap()));

    mock.expect_load_cell_data()
        .times(1)
        .returning(|_, _| Ok(10u128.to_le_bytes().to_vec()));

    mock
}

#[test]
fn test_mint_token() {
    let correct_sudt_code_hash = [103u8; 32];

    let mut mock = generate_mint_token_params();

    mock.expect_load_cell_type()
        .times(4)
        .returning(move |index, _| {
            if index == 0 || index == 1 {
                Ok(Some(
                    Script::new_builder()
                        .code_hash(correct_sudt_code_hash.clone().pack())
                        .build(),
                ))
            } else if index == 2 {
                Ok(Some(Script::default()))
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    _verify(mock);
}

#[test]
#[should_panic(expected = "mint more sudt than expected")]
fn test_mint_token_when_mint_more_sudt_than_expected() {
    let correct_sudt_code_hash = [103u8; 32];

    let mut mock = generate_mint_token_params();

    mock.expect_load_cell_type()
        .times(3)
        .returning(move |_, _| {
            Ok(Some(
                Script::new_builder()
                    .code_hash(correct_sudt_code_hash.clone().pack())
                    .build(),
            ))
        });

    _verify(mock);
}
