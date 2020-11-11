use super::*;
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};

const MAX_CYCLES: u64 = 10_000_000;

#[test]
fn test_mint_mode() {
    // deploy contract
    let mut context = Context::default();
    let bridge_lock: Bytes = Loader::default().load_binary("eth-bridge-lockscript");
    let bridge_lock_out_point = context.deploy_cell(bridge_lock);

    // prepare scripts
    let bridge_lock_script = context
        .build_script(&bridge_lock_out_point, Default::default())
        .expect("script");
    let bridge_lock_script_dep = CellDep::new_builder().out_point(bridge_lock_out_point).build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(bridge_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(bridge_lock_script.clone())
            .build(),
        CellOutput::new_builder().capacity(500u64.pack()).build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(bridge_lock_script_dep)
        .build();
    // let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
