use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_lock_hash, load_cell_type, load_script, load_script_hash, load_witness_args, QueryIter, load_input_out_point, load_cell};
use ckb_std::ckb_types::{
    packed::{Byte32, Script},
    prelude::Pack,
};
use molecule::prelude::Entity;
use molecule::bytes::Bytes;

// #[test]
// fn test_mint_mode_ok() {
//     let mut mock = MockAdapter::new();
//     mock.expect_load_input_data()
//         .times(1)
//         .returning(|| vec![]);
//     mock.expect_load_input_witness_args()
//         .times(1)
//         .returning(|| Ok(Bytes::from("aa")));
//     _verify(mock);
// }

// #[test]
// #[should_panic(expected="expected")]
// fn mock_return_err_when_input_is_none() {
//     let mut mock = MockAdapter::new();
//     mock.expect_load_input_output_data()
//         .times(1)
//         .returning(|| Ok(BridgeCellDataTuple(None, Some([0].to_vec()))));
//     _verify(mock);
// }

// #[test]
// #[should_panic(expected="expected")]
// fn mock_return_err_when_data_changed() {
//     let mut mock = MockAdapter::new();
//     mock.expect_load_input_output_data()
//         .times(1)
//         .returning(|| Ok(BridgeCellDataTuple(Some([0].to_vec()), Some([1].to_vec()))));
//     _verify(mock);
// }
