use crate::adapter::Adapter;

#[derive(Debug, Clone, Default)]
pub struct ETHReceiptInfo {
    token_amount: [u8; 32],
    token_address: [u8; 32],
    ckb_recipient_address: [u8; 32],
    replay_resist_cell_id: [u8; 32],
}

pub fn verify_mint_token<T: Adapter>(data_loader: T, input_data: &[u8], output_data: &[u8]) -> i8 {
    verify_data(input_data, output_data);
    verify_eth_light_client();
    let eth_receipt_info = verify_witness(data_loader);
    verify_eth_receipt_info(eth_receipt_info);
    0
}

pub fn verify_destroy_cell<T: Adapter>(_data_loader: T, _input_data: &[u8]) -> i8 {
    0
}

fn verify_data(input_data: &[u8], output_data: &[u8]) {
    if input_data != output_data {
        panic!("data changed")
    }

    // user pubkey is not none, should check signature
    if !input_data.is_empty() {
        verify_signature(input_data)
    }
}

fn verify_signature(_pubkey: &[u8]) {}

fn verify_eth_light_client() {}

/// Verify eth witness data.
/// 1. Verify that the header of the user's cross-chain tx is on the main chain.
/// 2. Verify that the user's cross-chain transaction is legal and really exists (based spv proof).
/// 3. Get ETHReceiptInfo from spv proof.
///
fn verify_witness<T: Adapter>(_data_loader: T) -> ETHReceiptInfo {
    ETHReceiptInfo::default()
}

/// Verify eth receipt info.
/// 1. Verify ckb_recipient_address get a number of token_amount cToken.
/// 2. Verify token_address equals to args.token_address.
/// 3. Verify replay_resist_cell_id exists in inputs.
fn verify_eth_receipt_info(_eth_receipt_info: ETHReceiptInfo) {}
