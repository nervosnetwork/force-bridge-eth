use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_types::packed::WitnessArgs;
use contracts_helper::data_loader::MockDataLoader;
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

#[test]
#[should_panic(expected = "proof witness lock field is none")]
fn test_mock_chain() {
    let mut mock_chain = MockDataLoader::new();
    mock_chain
        .expect_load_witness_args()
        .returning(|_index, _source| Ok(WitnessArgs::default()));
    let adapter = crate::adapter::ChainAdapter { chain: mock_chain };
    _verify(adapter);
}
