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
        script.merkle = vec!["height-2.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec!["height-2.json".to_string()];
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_init_batch_header() {
    let mut case = get_correct_case();

    case.script_cells.inputs = vec![];
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec!["height-2.json".to_string(), "height-3.json".to_string()];
        script.merkle = vec!["height-2.json".to_string(), "height-3.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec!["height-2.json".to_string(), "height-3.json".to_string()];
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
        witness.headers = vec!["height-3.json".to_string()];
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_batch_header() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
        ];
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
            "height-10917840.json".to_string(),
            "height-10917841.json".to_string(),
        ];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec![
            "height-10917839.json".to_string(),
            "height-10917840.json".to_string(),
            "height-10917841.json".to_string(),
        ];
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
        witness.headers = vec!["height-10913469-1.json".to_string()];
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
        script.merkle = vec!["height-10913469.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec!["height-10913469.json".to_string()];
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_reorg_batch_header() {
    let mut case = get_correct_case();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838-1.json".to_string(),
        ];
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = vec![
            "height-10917837.json".to_string(),
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
        ];
        script.uncle = vec!["height-10917838-1.json".to_string()];
        script.merkle = vec![
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
        ];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec![
            "height-10917838.json".to_string(),
            "height-10917839.json".to_string(),
        ];
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
        script.merkle = vec!["height-10913469-1.json".to_string()];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec!["height-10913469-1.json".to_string()];
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
            "height-10917840.json".to_string(),
            "height-10917841.json".to_string(),
        ];
        script.uncle = vec![
            "height-10917838-1.json".to_string(),
            "height-10917839-1.json".to_string(),
        ];
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec![
            "height-10917839.json".to_string(),
            "height-10917840.json".to_string(),
            "height-10917841.json".to_string(),
        ];
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
        witness.headers = vec!["height-10917845.json".to_string()];
    }

    case_runner::run_test(case);
}

#[test]
fn test_correct_tx_when_push_header_to_cache_limit() {
    let mut case = get_correct_case();

    let mut inputs = vec!["height-10917842.json".to_string(); 500];
    inputs[0] = "height-10917838.json".to_string();
    inputs[1] = "height-10917839.json".to_string();
    inputs[2] = "height-10917840.json".to_string();
    inputs[3] = "height-10917841.json".to_string();

    let mut outputs = vec!["height-10917842.json".to_string(); 500];
    outputs[0] = "height-10917841.json".to_string();
    outputs[497] = "height-10917843.json".to_string();
    outputs[498] = "height-10917844.json".to_string();
    outputs[499] = "height-10917845.json".to_string();

    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.inputs[0] {
        script.main = inputs;
    }
    if let CustomCell::ETHLightClientTypeCustomCell(script) = &mut case.script_cells.outputs[0] {
        script.main = outputs;
    }
    if let Witness::ETHLightClientWitness(witness) = &mut case.witnesses[0] {
        witness.headers = vec![
            "height-10917843.json".to_string(),
            "height-10917844.json".to_string(),
            "height-10917845.json".to_string(),
        ];
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
                    merkle: vec![],
                },
            )],
            outputs: vec![CustomCell::ETHLightClientTypeCustomCell(
                ETHLightClientTypeCell {
                    capacity: 100 * CKB_UNITS,
                    index: 0,
                    main: vec![],
                    uncle: vec![],
                    merkle: vec![],
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
            headers: vec![],
        })],
        expect_return_error_info: String::default(),
    }
}
