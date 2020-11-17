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

// eth-recipient-typescript has two situations based on whether outputs have eth-recipient-typescript data:
// 1: if outputs have data, we ensure it's a burn-token tx.
// 2: if outputs don't have data, it's a destroy eth-receipt-cell tx, it will always success.
pub fn _verify<T: Adapter>(data_loader: T) -> i8 {
    actions::verify_client_owner(data_loader);
    0
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::*;
    use ckb_std::ckb_types::packed::{Byte, Byte32, Bytes, Script};
    use molecule::prelude::{Builder, Entity};

    fn build_script(args: Bytes) -> Script {
        Script::new_builder()
            .hash_type(Byte::new(0))
            .code_hash(Byte32::default())
            .args(args)
            .build()
    }
    #[test]
    fn mock_return_ok_when_owner_is_none() {
        let input_script = build_script(Bytes::default());
        let output_script = build_script(Bytes::default());
        let mut mock = MockAdapter::new();
        mock.expect_load_input_script()
            .times(1)
            .returning(move || input_script.clone());
        mock.expect_load_output_script()
            .times(1)
            .returning(move || output_script.clone());
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_panic_when_change_owner_from_none_to_some() {
        let input_script = build_script(Bytes::default());
        let output_script = build_script(Bytes::new_builder().push(Byte::new(1)).build());
        let mut mock = MockAdapter::new();
        mock.expect_load_input_script()
            .times(1)
            .returning(move || input_script.clone());
        mock.expect_load_output_script()
            .times(1)
            .returning(move || output_script.clone());
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    fn mock_return_ok_when_change_owner_from_some_to_some() {
        let input_script = build_script(Bytes::new_builder().push(Byte::new(1)).build());
        let mut mock = MockAdapter::new();
        mock.expect_load_input_script()
            .times(1)
            .returning(move || input_script.clone());
        mock.expect_check_input_owner().times(1).returning(|_| true);
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_panic_when_owner_not_in_inputs() {
        let input_script = build_script(Bytes::new_builder().push(Byte::new(1)).build());
        let mut mock = MockAdapter::new();
        mock.expect_load_input_script()
            .times(1)
            .returning(move || input_script.clone());
        mock.expect_check_input_owner()
            .times(1)
            .returning(|_| false);
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
