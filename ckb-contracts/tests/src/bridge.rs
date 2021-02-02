use crate::utils::{case_builder::*, case_runner};
use ckb_tool::ckb_types::packed::Script;
use force_eth_types::config::CKB_UNITS;
use molecule::prelude::Entity;

#[test]
fn test_correct_tx() {
    let case = get_correct_case();
    case_runner::run_test(case);
}

// witness header number is 45
#[test]
fn test_tx_when_witness_header_is_not_confirmed() {
    let mut case = get_correct_case();
    let CellDepView::ETHBridgeLockCellDep(cell_dep) = &mut case.cell_deps[0];
    // replace block to block 40
    cell_dep.latest_height = 50;
    case.expect_return_error_info = "header is not confirmed".to_string();
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
        59, 189, 163, 204, 232, 1, 20, 0, 0, 0, 164, 191, 142, 76, 127, 111, 101, 243, 93, 211,
        204, 48, 200, 252, 69, 200, 233, 154, 23, 28,
    ])
    .unwrap();
    let always_success_lockscript = Script::from_slice(&[
        53u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 230, 131, 176, 65, 57, 52, 71, 104,
        52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0, 108, 13, 47, 236, 224, 10, 131,
        31, 54, 96, 215, 0, 0, 0, 0, 0,
    ])
    .unwrap();
    let correct_light_client_typescript_hash: [u8; 32] = [
        15, 239, 225, 21, 39, 234, 20, 233, 216, 42, 28, 161, 2, 142, 201, 234, 125, 126, 148, 115,
        19, 215, 118, 49, 98, 233, 91, 229, 67, 65, 9, 22,
    ];

    TestCase {
        cell_deps: vec![CellDepView::ETHBridgeLockCellDep(ETHBridgeLockDep {
            start_height: 0,
            latest_height: 100,
            merkle_root: [175u8, 67, 243, 141, 58, 48, 69, 47, 119, 171, 231, 65, 46, 177, 226, 106, 51, 80, 177, 154, 197, 96, 93, 198, 1, 140, 58, 88, 207, 8, 99, 82,],
        })],
        script_cells: CustomCells {
            inputs: vec![CustomCell::ETHBridgeCustomCell(ETHBridgeCell {
                capacity: 100 * CKB_UNITS,
                index: 0,
                eth_contract_address: "cD62E77cFE0386343c15C13528675aae9925D7Ae".to_string(),
                eth_token_address: "0000000000000000000000000000000000000000".to_string(),
                light_client_typescript_hash: correct_light_client_typescript_hash,
            })],
            outputs: vec![CustomCell::ETHBridgeCustomCell(ETHBridgeCell {
                capacity: 100 * CKB_UNITS,
                index: 2,
                eth_contract_address: "cD62E77cFE0386343c15C13528675aae9925D7Ae".to_string(),
                eth_token_address: "0000000000000000000000000000000000000000".to_string(),
                light_client_typescript_hash: correct_light_client_typescript_hash,
            })],
        },
        sudt_cells: SudtCells {
            inputs: vec![],
            outputs: vec![
                SudtCell {
                    capacity: 100 * CKB_UNITS,
                    amount: 92,
                    lockscript: recipient_lockscript,
                    owner_script: ScriptView::build_sudt_owner(
                        "cD62E77cFE0386343c15C13528675aae9925D7Ae",
                        "0000000000000000000000000000000000000000",
                        &correct_light_client_typescript_hash,
                    ),
                    index: 0,
                    sudt_extra_data: "sudt_extra_data".to_string(),
                },
                SudtCell {
                    capacity: 100 * CKB_UNITS,
                    amount: 8,
                    lockscript: always_success_lockscript,
                    owner_script: ScriptView::build_sudt_owner(
                        "cD62E77cFE0386343c15C13528675aae9925D7Ae",
                        "0000000000000000000000000000000000000000",
                        &correct_light_client_typescript_hash,
                    ),
                    index: 1,
                    sudt_extra_data: "sudt_extra_data".to_string(),
                },
            ],
        },
        capacity_cells: CapacityCells {
            inputs: vec![CapacityCell {
                capacity: 200 * CKB_UNITS,
                lockscript: ScriptView::default(),
                index: 1,
            }],
            outputs: vec![],
        },
        witnesses: vec![Witness::ETHBridgeWitness(ETHBridgeLockWitness {
            spv_proof: "f70800001800000020000000280000007903000095050000000000000000000000000000000000004d030000f9034a0182ba55b9010000000000000020100000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000410000020000000000000000000000000000000002000000000000000000000000000000000f90240f9023d94cd62e77cfe0386343c15c13528675aae9925d7aef863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024ce2af4461cc6062998febffea311866388e8c869af0cf89ce832dadcd3521f270000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f64617461000000000000000000000000000000000018020000f90215a036d2422d8abce3d513b6a0f20e7488b7279cedda3bd63ad0fc1638557203a621a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479417c4b5ce0605f63732bfd175fece7ac6b4620fd2a030c80a651cf02f1f53b298be2717084f760591ef0bbb9893840b213bf040de95a0ac6299aea5a7cd8c5a7f26e8c91551582dfd5a49f2e8ec2f4e29abeb7c150d0ca08bd3944f470118f235922d306a8a7f1d92cf4d1924bb30bf7f4f530728f7b72db901000000000000002010000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000002000000000000000000080000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000041000002000000000000000000000000000000000200000000000000000000000000000000083020a492d8657098f88691582ba558460099dcc99d883010917846765746888676f312e31352e33856c696e7578a016733dcbdcbef8e6591fb73c78e160af91811658cd49a589efac8f4b41f9fab08809058be5d324de32620300000800000056030000f90353822080b9034df9034a0182ba55b9010000000000000020100000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000410000020000000000000000000000000000000002000000000000000000000000000000000f90240f9023d94cd62e77cfe0386343c15c13528675aae9925d7aef863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000a4bf8e4c7f6f65f35dd3cc30c8fc45c8e99a171c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024ce2af4461cc6062998febffea311866388e8c869af0cf89ce832dadcd3521f270000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f646174610000000000000000000000000000000000".to_string(),
            block_number: 45,
            block_hash: "9485ad52a452389c67ea2364ae42c7a8772c54da7fa2929a6b2bf5261874298e".to_string()
        })],
        expect_return_error_info: String::default(),
    }
}
