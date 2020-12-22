use crate::adapter::Adapter;

use ckb_std::ckb_constants::Source;

/// 1. Verify args equal to first input's outpoint.
/// 2. Verify outputs only have 1 data cell.
pub fn verify<T: Adapter>(data_loader: T) {
    let input_data_len = data_loader.load_data_len_from_source(Source::GroupInput);
    if input_data_len == 0 {
        assert_eq!(
            data_loader.load_first_outpoint().as_ref(),
            data_loader.load_script_args().as_ref(),
            "invalid first cell id"
        )
    }

    let output_data_len = data_loader.load_data_len_from_source(Source::GroupOutput);

    assert!(output_data_len <= 1);
}
