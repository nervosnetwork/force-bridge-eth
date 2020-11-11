#![cfg_attr(not(feature = "std"), no_std)]

extern crate no_std_compat as std;

pub mod actions;
pub mod adapter;
pub mod debug;

use adapter::Adapter;
use adapter::BridgeCellDataTuple;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    let chain = adapter::chain::ChainAdapter {};
    _verify(chain);
    0
}

pub fn _verify<T: Adapter>(data_loader: T) {
    // let cell_data_tuple = data_loader
    //     .load_input_output_data()
    //     .expect("inputs or outputs cell num invalid");
    //
    // match cell_data_tuple {
    //     BridgeCellDataTuple(Some(input_data), Some(output_data)) => {
    //         actions::verify_mint_token(data_loader, input_data.as_slice(), output_data.as_slice())
    //     }
    //     // BridgeCellDataTuple(Some(input_data), None) => {
    //     //     actions::verify_destroy_cell(data_loader, input_data.as_slice())
    //     // }
    //     _ => panic!("input and output should not be none"),
    // }
    let mode = actions::check_mode(&data_loader);
    match mode {
        actions::Mode::Owner => {
            actions::verify_owner_mode(&data_loader);
        }
        actions::Mode::Mint => {
            actions::verify_mint_token(&data_loader);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::*;

    // #[test]
    // fn mock_return_ok() {
    //     let mut mock = MockAdapter::new();
    //     mock.expect_load_input_output_data()
    //         .times(1)
    //         .returning(|| Ok(BridgeCellDataTuple(Some([].to_vec()), Some([].to_vec()))));
    //     let return_code = _verify(mock);
    //     assert_eq!(return_code, 0);
    // }

    #[test]
    #[should_panic]
    fn mock_return_err_when_input_is_none() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok(BridgeCellDataTuple(None, Some([0].to_vec()))));
        _verify(mock);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_data_changed() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok(BridgeCellDataTuple(Some([0].to_vec()), Some([1].to_vec()))));
        _verify(mock);
    }
}
