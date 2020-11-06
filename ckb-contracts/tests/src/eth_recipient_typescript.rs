#[allow(unused_imports)]
use crate::utils::{case_builder::*, case_runner};
use force_eth_types::config::CKB_UNITS;

#[test]
fn test_correct_tx() {
    let case = get_correct_case();
    case_runner::run_test(case);
}

#[allow(dead_code)]
fn get_correct_case() -> TestCase {
    TestCase {
        cell_deps: vec![],
        script_cells: ScriptCellView::ETHRecipientScript(ETHRecipientCells {
            outputs: vec![ETHRecipientCell {
                capacity: 100 * CKB_UNITS,
                data: ETHRecipientDataView {
                    eth_recipient_address: "0x".to_string(),
                    eth_token_address: "0x1".to_string(),
                    token_amount: 0,
                },
                index: 0,
            }],
        }),
        sudt_cells: SudtCells {
            inputs: vec![SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 1,
                lockscript: Default::default(),
                owner_script: Default::default(),
                index: 1,
            }],
            outputs: vec![SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 1,
                lockscript: Default::default(),
                owner_script: Default::default(),
                index: 1,
            }],
        },
        capacity_cells: CapacityCells {
            inputs: vec![CapacityCell {
                capacity: 200 * CKB_UNITS,
                lockscript: Default::default(),
                index: 0,
            }],
            outputs: vec![],
        },
        witnesses: vec![],
        expect_return_code: 0,
    }
}
