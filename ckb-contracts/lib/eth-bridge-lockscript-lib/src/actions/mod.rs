use crate::adapter::Adapter;

pub fn verify_mint_token<T: Adapter>(data_loader: T) -> i8 {
    let data_list = data_loader.load_input_output_data().expect("inputs or outputs length invalid");
    if data_list.0 != data_list.1 {
        panic!("data changed")
    }
    0
}

#[cfg(test)]
mod tests {
    use super::verify_mint_token;
    use crate::adapter::*;
    use ckb_std::error::SysError;

    #[test]
    #[should_panic]
    fn mock_return_err() {
        let mut mock = MockAdapter::new();
        mock.expect_load_input_output_data()
            .times(1)
            .returning(|| Ok((Some([0].to_vec()), Some([1].to_vec()))));
        let return_code = verify_mint_token(mock);
        assert_eq!(return_code, 0);
    }
}
