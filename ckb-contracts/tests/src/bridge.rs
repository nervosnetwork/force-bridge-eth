use crate::utils::{case_builder::*, case_runner};
use ckb_tool::ckb_types::packed::Script;
use force_eth_types::config::CKB_UNITS;
use molecule::prelude::Entity;

#[test]
fn test_correct_tx() {
    let case = get_correct_case();
    case_runner::run_test(case);
}

// witness header number is 39, correct cell_dep.unconfirmed_blocks contains block 50
#[test]
fn test_tx_when_witness_header_is_not_confirmed() {
    let mut case = get_correct_case();
    if let CellDepView::ETHBridgeLockCellDep(cell_dep) = &mut case.cell_deps[0] {
        // replace block to block 40
        cell_dep.unconfirmed_blocks = vec!["f90216a06417bd3e58fbb42842be357b1b44a91a44b4907be271bb87990fd6015c99c0f7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794bb7b8287f3f0a933474a79eae42cbca977791171a058701736553780fd938e2195cbf4176cdf25935fae9c8e74d532bb3835aa37bea056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085040d13f2e228821388808455ba43419e476574682f4c5649562f76312e302e302f6c696e75782f676f312e342e32a0c8e64c566145f082ec79dfc40759b3e4b23b607a4c05e4b96cd5bfeb1222e754881953d8d473454660".to_string(); 10];
    }
    case.expect_return_error_info = "header is not confirmed".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_tx_when_witness_header_bigger_than_main_chain() {
    let mut case = get_correct_case();
    if let CellDepView::ETHBridgeLockCellDep(cell_dep) = &mut case.cell_deps[0] {
        // replace block to block 38
        cell_dep.unconfirmed_blocks = vec!["f90211a00ee49bf845e5d29a274bbab5b4ea7619a70b71e329b3c903236c3251d3f13e32a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347940193d941b50d91be6567c7ee1c0fe7af498b4137a0e615f6ef4bc5e58c2e387159763de9094f1ab6f60866be3ede684cff75e81dc1a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085040c10de7b26821388808455ba433d99476574682f76312e302e302f6c696e75782f676f312e342e32a0b5b623df7f772925a1447d7523a69a0faa10b6163b16a36919fcbfea1d1db64f88744f966546e1b351".to_string(); 10];
    }
    case.expect_return_error_info = "header is not on mainchain, header number too big".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_tx_when_witness_header_is_not_on_main_chain() {
    let mut case = get_correct_case();
    if let CellDepView::ETHBridgeLockCellDep(cell_dep) = &mut case.cell_deps[0] {
        cell_dep.confirmed_hashes = vec![];
    }
    case.expect_return_error_info =
        "header is not on mainchain, header number is too small".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_tx_when_witness_main_chain_contains_wrong_block_hash() {
    let mut case = get_correct_case();
    if let CellDepView::ETHBridgeLockCellDep(cell_dep) = &mut case.cell_deps[0] {
        cell_dep.confirmed_hashes = vec!["1234".to_string(); 10];
    }
    case.expect_return_error_info =
        "header is not on mainchain, target not in eth data".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_tx_when_sudt_extra_mismatch() {
    let mut case = get_correct_case();
    case.sudt_cells.outputs[0].sudt_extra_data = "test".to_string();
    case.expect_return_error_info = "recipient sudt cell extra data not match".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_tx_when_mint_wrong_total_amount() {
    let mut case = get_correct_case();
    case.sudt_cells.outputs[1].amount = 9;
    case.expect_return_error_info = "mint token amount not equal to expected".to_string();
    case_runner::run_test(case);
}

#[test]
fn test_tx_when_bridge_fee_is_less_than_expected() {
    let mut case = get_correct_case();
    case.sudt_cells.outputs[0].amount = 90;
    case.sudt_cells.outputs[1].amount = 10;
    case.expect_return_error_info =
        "recipient amount less than expected(mint_amount - bridge_fee)".to_string();
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
    let always_success_lockscript = Script::from_slice(&[
        53u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 230, 131, 176, 65, 57, 52, 71, 104,
        52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131,
        31, 54, 96, 215, 0, 0, 0, 0, 0,
    ])
    .unwrap();

    TestCase {
        cell_deps: vec![CellDepView::ETHBridgeLockCellDep(ETHBridgeLockDep {
            confirmed_hashes: vec!["ee7e2a1ea96119744c2965dcaf37954c0a7e9a6442d2057daae96a8d767c0ced".to_string(); 10],
            unconfirmed_blocks: vec!["f90216a0a87222c7d186888e978ece7e71a08497f3d02800b822b209a6fecf70d68510b1a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479417c4b5ce0605f63732bfd175fece7ac6b4620fd2a009ec189432b88128b963199e47e328a7046b69b375aeb807fff79bc8c0750dfda0a0b8c12f65e5209782bc177445a6c1c530b431c9725def3dc6c18cc2a3fc72baa0609cd096ce3e1858f8b32709efd2f3cc8d141afd6962efe7358e625c0c4d9e07b901000000000000000000000000000000000000000000001000000000000000000000001000000000000000000000000000000000100000100080000000100000000000000000000000000000000000000000104000000000000000000000100000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400010000000000000000000000000000000000800000000000000000000010000000000004000000000000000000000000000000002000040000000000000000000000000000000000000000000000000000000400000000800008000083020bcf3286569cf9ed6083830f8746845fcf554a99d883010917846765746888676f312e31352e33856c696e7578a0f7bb2320f060fd7a66c840ef2c90c6fc78a1b5b0c912615323da24e77db80c7d884d9fe7a2022a65ba".to_string(); 10],
        })],
        script_cells: CustomCells {
            inputs: vec![CustomCell::ETHBridgeCustomCell(ETHBridgeCell {
                capacity: 100 * CKB_UNITS,
                index: 0,
                eth_contract_address: "E9e7593081828a222E38E22578D9241D32504013".to_string(),
                eth_token_address: "0000000000000000000000000000000000000000".to_string(),
            })],
            outputs: vec![CustomCell::ETHBridgeCustomCell(ETHBridgeCell {
                capacity: 100 * CKB_UNITS,
                index: 2,
                eth_contract_address: "E9e7593081828a222E38E22578D9241D32504013".to_string(),
                eth_token_address: "0000000000000000000000000000000000000000".to_string(),
            })],
        },
        sudt_cells: SudtCells {
            inputs: vec![],
            outputs: vec![SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 92,
                lockscript: recipient_lockscript,
                owner_script: ScriptView::build_sudt_owner(
                    "E9e7593081828a222E38E22578D9241D32504013",
                    "0000000000000000000000000000000000000000",
                ),
                index: 0,
                sudt_extra_data: "sudt_extra_data".to_string(),
            },SudtCell {
                capacity: 100 * CKB_UNITS,
                amount: 8,
                lockscript: always_success_lockscript,
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
