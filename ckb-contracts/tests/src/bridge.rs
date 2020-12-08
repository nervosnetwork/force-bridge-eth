use crate::utils::{case_builder::*, case_runner};
use ckb_tool::ckb_types::packed::Script;
use force_eth_types::config::CKB_UNITS;
use molecule::prelude::Entity;

#[test]
fn test_correct_tx() {
    let case = get_correct_case();
    case_runner::run_test(case);
}

fn get_correct_case() -> TestCase {
    let recipient_lockscript = Script::from_slice(&[
        73u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 155, 215, 224, 111, 62, 207, 75, 224,
        242, 252, 210, 24, 139, 35, 241, 185, 252, 200, 142, 93, 75, 101, 168, 99, 123, 23, 114,
        59, 189, 163, 204, 232, 1, 20, 0, 0, 0, 71, 13, 205, 197, 228, 64, 100, 144, 150, 80, 17,
        58, 39, 75, 59, 54, 174, 203, 109, 199,
    ])
    .unwrap();

    TestCase {
        cell_deps: vec![CellDepView::ETHBridgeLockCellDep(ETHBridgeLockDep {
            confirmed_hashes: vec!["ee7e2a1ea96119744c2965dcaf37954c0a7e9a6442d2057daae96a8d767c0ced".to_string(); 10],
            unconfirmed_blocks: vec!["f90216a0a87222c7d186888e978ece7e71a08497f3d02800b822b209a6fecf70d68510b1a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479417c4b5ce0605f63732bfd175fece7ac6b4620fd2a009ec189432b88128b963199e47e328a7046b69b375aeb807fff79bc8c0750dfda0a0b8c12f65e5209782bc177445a6c1c530b431c9725def3dc6c18cc2a3fc72baa0609cd096ce3e1858f8b32709efd2f3cc8d141afd6962efe7358e625c0c4d9e07b901000000000000000000000000000000000000000000001000000000000000000000001000000000000000000000000000000000100000100080000000100000000000000000000000000000000000000000104000000000000000000000100000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400010000000000000000000000000000000000800000000000000000000010000000000004000000000000000000000000000000002000040000000000000000000000000000000000000000000000000000000400000000800008000083020bcf3286569cf9ed6083830f8746845fcf554a99d883010917846765746888676f312e31352e33856c696e7578a0f7bb2320f060fd7a66c840ef2c90c6fc78a1b5b0c912615323da24e77db80c7d884d9fe7a2022a65ba".to_string(); 10],
        })],
        script_cells: CustomCells {
            inputs: vec![CustomCell::ETHBridgeLockCustomCell(ETHBridgeLockCell {
                capacity: 100 * CKB_UNITS,
                index: 0,
                eth_contract_address: "E9e7593081828a222E38E22578D9241D32504013".to_string(),
                eth_token_address: "0000000000000000000000000000000000000000".to_string(),
            })],
            outputs: vec![CustomCell::ETHBridgeLockCustomCell(ETHBridgeLockCell {
                capacity: 100 * CKB_UNITS,
                index: 0,
                eth_contract_address: "E9e7593081828a222E38E22578D9241D32504013".to_string(),
                eth_token_address: "0000000000000000000000000000000000000000".to_string(),
            })],
        },
        sudt_cells: SudtCells {
            inputs: vec![],
            outputs: vec![SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 100,
                lockscript: recipient_lockscript,
                owner_script: ScriptView::build_sudt_owner(
                    "E9e7593081828a222E38E22578D9241D32504013",
                    "0000000000000000000000000000000000000000",
                ),
                index: 1,
                sudt_extra_data: "sudt_extra_data".to_string(),
            }],
        },
        capacity_cells: CapacityCells {
            inputs: vec![CapacityCell {
                capacity: 200 * CKB_UNITS,
                lockscript: ScriptView::default(),
                index: 1,
            }],
            outputs: vec![],
        },
        witnesses: vec![Witness::ETHBridgeWitness(ETHBridgeLockWitness {})],
        expect_return_error_info: String::default(),
    }
}
