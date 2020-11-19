pub mod types;
pub mod utils;

use crate::utils::{case_builder::*, case_runner};
use force_eth_types::config::CKB_UNITS;

#[test]
fn test_correct_tx_when_init_header() {
    let mut case = get_correct_case();

    case.script_cells.inputs = vec![];
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec!["height-2.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-2.json".to_string();
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_header() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec!["height-2.json".to_string()];
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec!["height-2.json".to_string(), "height-3.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-3.json".to_string();
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_uncle_to_main() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec!["height-10913468.json".to_string()];
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10913468.json".to_string(),
            "height-10913469-1.json".to_string(),
        ];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-10913469-1.json".to_string();
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_header_reorg() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec![
            "height-10913468.json".to_string(),
            "height-10913469-1.json".to_string(),
        ];
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10913468.json".to_string(),
            "height-10913469.json".to_string(),
        ];
        script.uncle = vec!["height-10913469-1.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-10913469.json".to_string();
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_uncle() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec![
            "height-10913468.json".to_string(),
            "height-10913469.json".to_string(),
        ];
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10913468.json".to_string(),
            "height-10913469.json".to_string(),
        ];
        script.uncle = vec!["height-10913469-1.json".to_string()];
        script.merkle = Some("height-10913469-1.json".to_string());
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-10913469-1.json".to_string();
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_uncle_block_and_reorg() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
            "height-10917839-1.json".to_string(),
        ];
        script.uncle = vec!["height-10917838-1.json".to_string()]
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
        ];
        script.uncle = vec![
            "height-10917838-1.json".to_string(),
            "height-10917839-1.json".to_string(),
        ];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-10917839.json".to_string();
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_header_reorg_and_update_uncles() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
            "height-10917840.json".to_string(),
            "height-10917841.json".to_string(),
            "height-10917842.json".to_string(),
            "height-10917843.json".to_string(),
            "height-10917844.json".to_string(),
            "height-10917845-1.json".to_string(),
        ];
        script.uncle = vec![
            "height-10917838-1.json".to_string(),
            "height-10917839-1.json".to_string(),
        ]
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
            "height-10917840.json".to_string(),
            "height-10917841.json".to_string(),
            "height-10917842.json".to_string(),
            "height-10917843.json".to_string(),
            "height-10917844.json".to_string(),
            "height-10917845.json".to_string(),
        ];
        script.uncle = vec![
            "height-10917838-1.json".to_string(),
            "height-10917839-1.json".to_string(),
            "height-10917845-1.json".to_string(),
        ];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.header = "height-10917845.json".to_string();
    }

    case_runner::run_test(case);
}

fn get_correct_case() -> TestCase {
    TestCase {
        cell_deps: vec![CellDepView::ETHLightClientTypeCellDep(
            ETHLightClientTypeDep {},
        )],
        script_cells: CustomCells {
            inputs: vec![CustomCell::ETHLightClientTypeCustomCell(
                ETHLightClientTypeCell {
                    capacity: 100 * CKB_UNITS,
                    index: 1,
                    main: vec![],
                    uncle: vec![],
                    merkle: None,
                },
            )],
            outputs: vec![CustomCell::ETHLightClientTypeCustomCell(
                ETHLightClientTypeCell {
                    capacity: 100 * CKB_UNITS,
                    index: 0,
                    main: vec![],
                    uncle: vec![],
                    merkle: None,
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
                index: 0,
            }],
            outputs: vec![],
        },
        witnesses: vec![Witness::ETHLightClientWitness(ETHLightClientTypeWitness {
            cell_dep_index_list: vec![0],
            header: String::default(),
        })],
        expect_return_error_info: String::default(),
    }
}
