use crate::adapter::Adapter;

use ckb_std::ckb_constants::Source;

/// 1. Verify args equal to first input's outpoint.
/// 2. Verify inputs and outputs only have 1 data cell.
pub fn verify<T: Adapter>(data_loader: T) {
    let input_data = data_loader.load_data_from_source(Source::GroupInput);
    if input_data.is_none() {
        assert_eq!(
            data_loader.load_first_outpoint().to_vec(),
            data_loader.load_script_args().to_vec(),
            "invalid first cell id"
        )
    }

    data_loader
        .load_data_from_source(Source::GroupOutput)
        .expect("output should not be none");
}
