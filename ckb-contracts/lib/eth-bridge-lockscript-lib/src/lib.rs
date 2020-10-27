#![cfg_attr(not(feature = "std"), no_std)]

mod actions;
pub mod adapter;
pub mod debug;

pub use adapter::Adapter;

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
    let input_output_num = data_loader.load_input_output_cell_num();
    match input_output_num {
        (1, 1) => actions::verify_mint_token(data_loader),
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
        mock.expect_load_input_output_cell_num()
            .times(1)
            .returning(|| (1, 1));
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok((Some([0].to_vec()), Some([0].to_vec()))));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_cell_num()
            .times(1)
            .returning(|| (0, 1));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
