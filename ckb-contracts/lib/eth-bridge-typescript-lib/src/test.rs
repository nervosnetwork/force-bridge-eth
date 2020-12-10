use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::Script;
use force_eth_types::generated::basic::Byte32;
use force_eth_types::generated::eth_bridge_lock_cell::ETHBridgeLockArgs;
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

    mock.expect_load_script_args()
        .times(1)
        .returning(move || ETHBridgeTypeArgs::default());

    mock.expect_load_data()
        .times(1)
        .returning(move || Some(ETHBridgeTypeData::default()));

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

#[test]
fn test_mint_token() {
    /// assume correct input bridge lock hash is [100u8; 32]
    /// assume correct input bridge type hash is [300u8; 32]
    /// assume sudt code hash is [1000u8;32]
    let mut mock = MockAdapter::new();

    mock.expect_load_input_witness_args()
        .times(1)
        .returning(move || Ok(MintTokenWitness::default().as_bytes()));

    mock.expect_load_script_args().times(1).returning(move || {
        ETHBridgeTypeArgs::new_builder()
            .bridge_lock_hash([100u8; 32].pack().into())
            .build()
    });

    mock.expect_load_data()
        .times(1)
        .returning(move || Some(ETHBridgeTypeData::default()));

    mock.expect_load_cell_lock_hash()
        .times(2)
        .returning(|_, source| {
            if source == Source::Input {
                Ok([100u8; 32])
            } else {
                Ok([200u8; 32])
            }
        });

    mock.expect_load_cell_type_hash()
        .times(1)
        .returning(|_, _| Ok(Some([300u8; 32])));

    mock.expect_load_script_hash()
        .times(1)
        .returning(|_, _| Ok([300u8; 32]));

    mock.expect_get_associated_udt_script()
        .times(1)
        .returning(|_| Script::new_builder().code_hash([1000u8; 32].pack().into()));

    mock.expect_load_cell_type()
        .times(1)
        .returning(|_| Script::new_builder().code_hash([1000u8; 32].pack().into()));

    _verify(mock);
}
