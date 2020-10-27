use crate::adapter::Adapter;

pub fn verify_mint_token<T: Adapter>(_data_loader: T, input_data: &[u8], output_data: &[u8]) -> i8 {
    if input_data != output_data {
        panic!("data changed")
    }
    0
}
