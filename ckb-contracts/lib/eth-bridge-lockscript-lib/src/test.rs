use crate::_verify;
use crate::adapter::*;
use force_eth_types::generated::witness::MintTokenWitness;
use molecule::prelude::{Builder, Entity};

#[test]
#[should_panic(expected = "eth spv proof is invalid")]
fn test_mint_mode_invalid_proof() {
    let mut mock = MockAdapter::new();
    let witness = MintTokenWitness::new_builder().build();
    mock.expect_load_input_witness_args()
        .times(1)
        .returning(move || Ok(witness.as_bytes()));
    _verify(mock);
}
