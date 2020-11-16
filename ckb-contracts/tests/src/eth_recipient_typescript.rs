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
                    eth_recipient_address: "5Dc158c90EBE46FfC9f03f1174f36c44497976D4".to_string(),
                    eth_token_address: "e404831459e3aCec0440F5c5462827e0Bccc2Ff1".to_string(),
                    token_amount: 100,
                    fee: 10,
                },
                index: 0,
                args: "74381D4533cc43121abFef7566010dD9FB7c9F7a".to_string(),
            }],
        }),
        sudt_cells: SudtCells {
            inputs: vec![SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 200,
                lockscript: Default::default(),
                owner_script: ScriptView::build_sudt_owner(
                    "74381D4533cc43121abFef7566010dD9FB7c9F7a",
                    "e404831459e3aCec0440F5c5462827e0Bccc2Ff1",
                ),
                index: 1,
            }],
            outputs: vec![SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 100,
                lockscript: Default::default(),
                owner_script: ScriptView::build_sudt_owner(
                    "74381D4533cc43121abFef7566010dD9FB7c9F7a",
                    "e404831459e3aCec0440F5c5462827e0Bccc2Ff1",
                ),
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
