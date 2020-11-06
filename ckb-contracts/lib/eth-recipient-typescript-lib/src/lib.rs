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
    let data = data_loader.load_output_data();
    if let Some(data) = data {
        actions::verify_burn_token(data_loader, data)
    }
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
        let data = ETHRecipientDataView {
            eth_recipient_address: Bytes::from([0u8].to_vec()),
            eth_token_address: Bytes::from([0u8].to_vec()),
            token_amount: 1,
        };
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

    #[test]
    #[should_panic]
    fn mock_return_err_when_input_less_than_output() {
        let data = ETHRecipientDataView {
            eth_recipient_address: Bytes::from([0u8].to_vec()),
            eth_token_address: Bytes::from([0u8].to_vec()),
            token_amount: 1,
        };
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 99 } else { 100 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_burned_amount_not_equal_data_amount() {
        let data = ETHRecipientDataView {
            eth_recipient_address: Bytes::from([0u8].to_vec()),
            eth_token_address: Bytes::from([0u8].to_vec()),
            token_amount: 1,
        };
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 100 } else { 98 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
