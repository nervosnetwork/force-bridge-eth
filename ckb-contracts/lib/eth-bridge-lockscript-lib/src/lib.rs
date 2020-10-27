#![cfg_attr(not(feature = "std"), no_std)]

mod actions;
pub mod adapter;
pub mod debug;

pub use adapter::Adapter;

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
    _verify(chain)
}

pub fn _verify<T: Adapter>(data_loader: T) -> i8 {
    let cell_data_tuple = data_loader
        .load_input_output_data()
        .expect("inputs or outputs cell num invalid");
    match cell_data_tuple {
        BridgeCellDataTuple(Some(input_data), Some(output_data)) => {
            actions::verify_mint_token(data_loader, input_data.as_slice(), output_data.as_slice())
        }
        _ => panic!("input and output should not be none"),
    }
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::*;
    use ckb_std::error::SysError;

    #[test]
    fn mock_return_ok() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok(BridgeCellDataTuple(Some([0].to_vec()), Some([0].to_vec()))));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_input_is_none() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok(BridgeCellDataTuple(None, Some([0].to_vec()))));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_data_changed() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok(BridgeCellDataTuple(Some([0].to_vec()), Some([1].to_vec()))));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
