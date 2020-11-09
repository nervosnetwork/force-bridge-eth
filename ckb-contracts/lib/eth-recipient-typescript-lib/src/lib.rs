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
    use core::convert::TryFrom;
    use force_eth_types::eth_recipient_cell::{ETHAddress, ETHRecipientDataView};
    use molecule::bytes::Bytes;

    #[test]
    fn mock_return_ok() {
        let data = ETHRecipientDataView {
            eth_recipient_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            eth_token_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            token_amount: 100,
            fee: 1,
        };
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8; 20].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 1000 } else { 900 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_input_less_than_output() {
        let data = ETHRecipientDataView {
            eth_recipient_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            eth_token_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            token_amount: 100,
            fee: 1,
        };
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8; 20].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 900 } else { 1000 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_burned_amount_not_equal_data_amount() {
        let data = ETHRecipientDataView {
            eth_recipient_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            eth_token_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            token_amount: 100,
            fee: 1,
        };
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8; 20].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 1000 } else { 800 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_fee_is_too_much() {
        let data = ETHRecipientDataView {
            eth_recipient_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            eth_token_address: ETHAddress::try_from(vec![0; 20]).unwrap(),
            token_amount: 100,
            fee: 100,
        };
        let mut mock = MockAdapter::new();
        mock.expect_load_output_data()
            .times(1)
            .returning(move || Some(data.clone()));
        mock.expect_load_script_args()
            .times(1)
            .returning(|| Bytes::from([0u8; 20].to_vec()));
        mock.expect_get_sudt_amount_from_source()
            .times(2)
            .returning(|x, _y| if x == Source::Input { 1000 } else { 900 });
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
