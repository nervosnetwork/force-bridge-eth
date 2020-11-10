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
    use ckb_std::ckb_constants::Source;
    use force_eth_types::eth_recipient_cell::ETHRecipientDataView;
    use molecule::bytes::Bytes;

    #[test]
    fn mock_return_ok() {
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 100 } else { 99 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
