use crate::utils::{case_builder::*, case_runner};
use force_eth_types::config::CKB_UNITS;

#[test]
fn test_correct_tx_when_in_owner_mode() {
    let case = get_correct_case();
    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_change_owner() {
    let mut case = get_correct_case();
    if let CustomCell::ETHLightClientLockCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.args[0] = 0;
    }
    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_change_owner_to_none() {
    let mut case = get_correct_case();
    if let CustomCell::ETHLightClientLockCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.args = Vec::default();
    }
    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_owner_from_none_to_none() {
    let mut case = get_correct_case();
    if let CustomCell::ETHLightClientLockCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.args = Vec::default();
    }
    if let CustomCell::ETHLightClientLockCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.args = Vec::default();
    }
    case_runner::run_test(case);
}

#[test]
fn test_wrong_tx_when_owner_from_none_to_some() {
    let mut case = get_correct_case();
    if let CustomCell::ETHLightClientLockCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.args = Vec::default();
    }
    case.expect_return_error_info = "owner changed from none to some".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_wrong_tx_when_not_in_owner_mode() {
    let mut case = get_correct_case();
    if let CustomCell::ETHLightClientLockCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.args[0] = 0;
    }
    case.expect_return_error_info = "not owner lock".to_string();

    case_runner::run_test(case);
}

fn get_correct_case() -> TestCase {
    let always_success_hash: Vec<u8> = [
        8, 142, 54, 93, 30, 19, 242, 26, 170, 49, 221, 68, 22, 77, 231, 167, 164, 85, 230, 41, 131,
        114, 38, 229, 3, 201, 92, 8, 198, 34, 61, 7,
    ]
    .to_vec();
    TestCase {
        cell_deps: vec![],
        script_cells: CustomCells {
            inputs: vec![CustomCell::ETHLightClientLockCustomCell(
                ETHLightClientLockCell {
                    capacity: 100 * CKB_UNITS,
                    index: 0,
                    args: always_success_hash.clone(),
                },
            )],
            outputs: vec![CustomCell::ETHLightClientLockCustomCell(
                ETHLightClientLockCell {
                    capacity: 100 * CKB_UNITS,
                    index: 0,
                    args: always_success_hash,
                },
            )],
        },
        sudt_cells: SudtCells {
            inputs: vec![],
            outputs: vec![],
        },
        capacity_cells: CapacityCells {
            inputs: vec![CapacityCell {
                capacity: 200 * CKB_UNITS,
                lockscript: ScriptView::default(),
                index: 1,
            }],
            outputs: vec![],
        },
        witnesses: vec![],
        expect_return_error_info: String::default(),
    }
}
