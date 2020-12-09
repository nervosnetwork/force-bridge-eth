#![cfg_attr(not(feature = "std"), no_std)]

pub mod actions;
pub mod adapter;
pub mod debug;

use crate::adapter::load_output_data;
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
    let data = load_output_data(&data_loader);
    if let Some(data) = data {
        actions::verify_burn_token(data_loader, data)
    }
    0
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::*;

    #[test]
    fn mock_return_ok() {
        let token_amount: u128 = 10;
        let fee: u128 = 1;
        let input_sudt_cell_data =
            "64000000000000000000000000000000737564745f65787472615f64617461".to_string();
        let output_sudt_cell_data = "5a000000000000000000000000000000".to_string();
        let (mol_data_vec, lock_hash) = get_mock_load_output_data(token_amount, fee);

        let mut mock = MockAdapter::new();
        mock = set_mock_chain_data(
            mock,
            mol_data_vec,
            lock_hash,
            input_sudt_cell_data,
            output_sudt_cell_data,
        );
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_input_less_than_output() {
        let token_amount: u128 = 10;
        let fee: u128 = 1;
        let (mol_data_vec, lock_hash) = get_mock_load_output_data(token_amount, fee);
        let input_sudt_cell_data = "5a000000000000000000000000000000".to_string();
        let output_sudt_cell_data = "64000000000000000000000000000000".to_string();
        let mut mock = MockAdapter::new();
        mock = set_mock_chain_data(
            mock,
            mol_data_vec,
            lock_hash,
            input_sudt_cell_data,
            output_sudt_cell_data,
        );
        _verify(mock);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_burned_amount_not_equal_data_amount() {
        let token_amount: u128 = 10;
        let fee: u128 = 1;
        let (mol_data_vec, lock_hash) = get_mock_load_output_data(token_amount, fee);
        let input_sudt_cell_data = "4a000000000000000000000000000000".to_string();
        let output_sudt_cell_data = "64000000000000000000000000000000".to_string();
        let mut mock = MockAdapter::new();
        mock = set_mock_chain_data(
            mock,
            mol_data_vec,
            lock_hash,
            input_sudt_cell_data,
            output_sudt_cell_data,
        );
        _verify(mock);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_fee_is_too_much() {
        let token_amount: u128 = 10;
        let fee: u128 = 100;
        let (mol_data_vec, lock_hash) = get_mock_load_output_data(token_amount, fee);
        let input_sudt_cell_data = "64000000000000000000000000000000".to_string();
        let output_sudt_cell_data = "5a000000000000000000000000000000".to_string();
        let mut mock = MockAdapter::new();
        mock = set_mock_chain_data(
            mock,
            mol_data_vec,
            lock_hash,
            input_sudt_cell_data,
            output_sudt_cell_data,
        );
        _verify(mock);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_lock_hash_wrong() {
        let token_amount: u128 = 10;
        let fee: u128 = 1;
        let input_sudt_cell_data =
            "64000000000000000000000000000000737564745f65787472615f64617461".to_string();
        let output_sudt_cell_data = "5a000000000000000000000000000000".to_string();
        let (mol_data_vec, _lock_hash) = get_mock_load_output_data(token_amount, fee);
        let lock_hash = [0u8; 32];

        let mut mock = MockAdapter::new();
        mock = set_mock_chain_data(
            mock,
            mol_data_vec,
            lock_hash,
            input_sudt_cell_data,
            output_sudt_cell_data,
        );
        _verify(mock);
    }
    #[test]
    #[should_panic]
    fn mock_return_err_when_cell_data_wrong() {
        let wrong_cell_data = hex::decode("5a0000000000").unwrap();
        let mol_data_vec = vec![wrong_cell_data];

        let mut mock = MockAdapter::new();
        mock.expect_load_output_data_by_trait()
            .times(1)
            .returning(move || mol_data_vec.clone());
        _verify(mock);
    }

    #[test]
    #[should_panic]
    fn mock_return_err_when_cell_type_return_err() {
        let token_amount: u128 = 10;
        let fee: u128 = 1;
        let (mol_data_vec, lock_hash) = get_mock_load_output_data(token_amount, fee);

        let mut mock = MockAdapter::new();
        mock.expect_load_output_data_by_trait()
            .times(1)
            .returning(move || mol_data_vec.clone());
        mock.expect_load_cell_type_by_trait()
            .times(2)
            .returning(move |_, _| get_mock_load_cell_type(CellType::OtherErr, &lock_hash));
        _verify(mock);
    }
}
