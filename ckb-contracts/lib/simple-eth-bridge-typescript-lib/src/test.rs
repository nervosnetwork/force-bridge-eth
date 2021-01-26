use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_types::packed::Script;
use ckb_std::ckb_types::prelude::Pack;
use ckb_std::error::SysError;
use contracts_helper::data_loader::MockDataLoader;
use molecule::prelude::{Builder, Entity};

struct TestParams {
    owner_script: Script,
    owner_lock_hash: [u8; 32],
}

fn get_correct_params() -> TestParams {
    let owner_script = Script::new_builder()
        .args([1u8; 32].as_ref().pack())
        .build();
    let owner_lock_hash = [1u8; 32];
    TestParams {
        owner_script,
        owner_lock_hash,
    }
}

fn generate_manage_mode_correct_mock(test_params: TestParams) -> MockDataLoader {
    let mut mock = MockDataLoader::new();

    let owner_lock_hash = test_params.owner_lock_hash;
    mock.expect_load_cell_lock_hash()
        .times(1)
        .returning(move |_, _| Ok(owner_lock_hash.clone()));

    let script = test_params.owner_script;
    mock.expect_load_script()
        .times(1)
        .returning(move || Ok(script.clone()));

    mock
}

#[test]
fn test_correct_manage_mode() {
    let test_params = get_correct_params();
    let mock = generate_manage_mode_correct_mock(test_params);

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
#[should_panic(expected = "not authorized to unlock the cell")]
fn test_manage_mode_when_lock_script_not_exist_in_inputs() {
    let mut test_params = get_correct_params();
    test_params.owner_lock_hash = [0u8; 32];

    let mut mock = generate_manage_mode_correct_mock(test_params);

    mock.expect_load_cell_lock_hash()
        .times(1)
        .returning(move |_, _| Err(SysError::IndexOutOfBound));

    let adapter = ChainAdapter { chain: mock };

    _verify(adapter);
}
