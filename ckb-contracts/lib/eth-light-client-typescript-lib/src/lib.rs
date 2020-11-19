#![cfg_attr(not(feature = "std"), no_std)]

pub mod actions;
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
    actions::verify_add_headers(data_loader);
    0
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::*;

    #[test]
    #[should_panic]
    fn mock_panic_when_input_and_output_is_none() {
        let mut mock = MockAdapter::new();
        mock.expect_load_data_from_source()
            .times(2)
            .returning(|_| None);
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
