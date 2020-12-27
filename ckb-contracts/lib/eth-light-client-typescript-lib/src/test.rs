use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::{OutPoint, Script};
use ckb_std::ckb_types::prelude::Pack;
use ckb_std::error::SysError;
use contracts_helper::data_loader::MockDataLoader;
use molecule::prelude::{Builder, Entity};

fn generate_init_correct_mock() -> MockDataLoader {
    let mut mock = MockDataLoader::new();

    mock.expect_load_cell_data()
        .times(3)
        .returning(move |index, source| {
            if source == Source::GroupOutput {
                if index == 0 {
                    Ok(Default::default())
                } else {
                    Err(SysError::IndexOutOfBound)
                }
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    mock.expect_load_input_out_point()
        .times(1)
        .returning(|_, _| {
            Ok(OutPoint::new_builder()
                .index(0u32.pack())
                .tx_hash([1u8; 32].pack())
                .build())
        });

    mock
}

fn generate_push_correct_mock() -> MockDataLoader {
    let mut mock = MockDataLoader::new();

    mock.expect_load_cell_data()
        .times(4)
        .returning(move |index, source| {
            if index == 0 {
                Ok(Default::default())
            } else {
                Err(SysError::IndexOutOfBound)
            }
        });

    mock
}

#[test]
fn test_init_client_correct() {
    let mut mock = generate_init_correct_mock();

    mock.expect_load_script().times(1).returning(|| {
        Ok(Script::new_builder()
            .args(
                [
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
                ]
                .as_ref()
                .pack(),
            )
            .build())
    });

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
#[should_panic(expected = "invalid first cell id")]
fn test_init_client_wrong_when_args_first_input_outpoint_wrong() {
    let mut mock = generate_init_correct_mock();

    mock.expect_load_script()
        .times(1)
        .returning(|| Ok(Script::new_builder().args([1u8].pack()).build()));

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
fn test_push_client_correct() {
    let mock = generate_push_correct_mock();

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}
