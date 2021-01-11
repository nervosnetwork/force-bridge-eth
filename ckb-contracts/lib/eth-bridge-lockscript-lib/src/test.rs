use crate::_verify;
use crate::adapter::*;
use ckb_std::ckb_types::packed::{self, CellOutput, OutPoint, Script, WitnessArgs};
use ckb_std::ckb_types::prelude::Pack;
use ckb_std::error::SysError;
use contracts_helper::data_loader::MockDataLoader;
use force_eth_types::config::{SUDT_CODE_HASH, SUDT_HASH_TYPE};
use force_eth_types::eth_recipient_cell::ETHAddress;
use force_eth_types::generated::{
    basic, eth_bridge_lock_cell::ETHBridgeLockArgs, eth_header_cell::ETHHeaderCellData,
    witness::MintTokenWitness,
};
use force_eth_types::hasher::Blake2bHasher;
use molecule::bytes::Bytes;
use molecule::prelude::{Builder, Byte, Entity};
use sparse_merkle_tree::{default_store::DefaultStore, SparseMerkleTree, H256};
use std::convert::TryFrom;

type SMT = SparseMerkleTree<Blake2bHasher, H256, DefaultStore<H256>>;

fn generate_mint_mode_mock(lock_script: Script) -> MockDataLoader {
    let mut mock = MockDataLoader::new();

    let witness_args = generate_mint_token_witness();
    mock.expect_load_witness_args()
        .times(1)
        .returning(move |_, _| Ok(witness_args.clone()));

    let light_client_data = generate_light_client_data();
    mock.expect_load_cell_data()
        .times(1)
        .returning(move |_, _| Ok(light_client_data.clone().to_vec()));

    let correct_input_outpoint =
        "5f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc94482200000000";
    let outpoint =
        OutPoint::from_slice(hex::decode(correct_input_outpoint).unwrap().as_slice()).unwrap();
    mock.expect_load_input_out_point()
        .times(1)
        .returning(move |_, _| Ok(outpoint.clone()));

    mock.expect_load_script()
        .times(1)
        .returning(move || Ok(lock_script.clone()));

    mock.expect_load_cell_type_hash()
        .times(1)
        .returning(|_, _| Ok(Some([1u8; 32])));

    mock.expect_load_script_hash()
        .times(1)
        .returning(|| Ok([2u8; 32]));

    let (cell, sudt_data) = generate_sudt_cell();
    mock.expect_load_cell().times(2).returning(move |index, _| {
        if index == 0 {
            Ok(cell.clone())
        } else {
            Err(SysError::IndexOutOfBound)
        }
    });

    mock.expect_load_cell_data()
        .times(1)
        .returning(move |_, _| Ok(sudt_data.clone()));

    mock
}

fn generate_smt_tree() -> SMT {
    let mut smt_tree = SMT::default();

    for i in 0u8..100 {
        let mut key = [0u8; 32];
        key[0] = i.to_le_bytes()[0];
        let mut value = [i; 32];

        if i == 39 {
            let mut block_hash = [0u8; 32];
            block_hash.copy_from_slice(
                hex::decode("ee7e2a1ea96119744c2965dcaf37954c0a7e9a6442d2057daae96a8d767c0ced")
                    .unwrap()
                    .as_slice(),
            );
            value = block_hash;
        }

        smt_tree.update(key.into(), value.into()).unwrap();
    }
    smt_tree
}

fn generate_mint_token_witness() -> WitnessArgs {
    let correct_spv_proof = "3f0b00001c000000240000006802000070020000c1050000dd070000000000000000000040020000f9023d94e9e7593081828a222e38e22578d9241d32504013f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000470dcdc5e44064909650113a274b3b36aecb6dc7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000245f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc9448220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f64617461000000000000000000000000000000000000000000000000004d030000f9034a01828005b9010000000000000020100000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000400000000000000000000008000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000020000000000000000000000000000000002000000000000000000000000000000000f90240f9023d94e9e7593081828a222e38e22578d9241d32504013f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000470dcdc5e44064909650113a274b3b36aecb6dc7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000245f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc9448220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f64617461000000000000000000000000000000000018020000f90215a04809bc46bf97f29706c0ecf624e4c1365a031afd0956dd2442e870611a39dcdaa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d493479417c4b5ce0605f63732bfd175fece7ac6b4620fd2a074ecc5c6946ccf5216e5ac05388aa24685ea3763f19d203c4a6516ac20f25faea03a0eee4b28ff7e6da2cf5a78c00c1115e3c452162bc2639a9da749605596b2e4a0d3b519ded443cf60121272d2eef6705f5c45c068f02603ee65d61b111e0ea958b9010000000000000020100000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000400000000000000000000008000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000020000000000000000000000000000000002000000000000000000000000000000000830209042786578c90689556828005845fcf552b99d883010917846765746888676f312e31352e33856c696e7578a0d67a513647fc02e9e29f8b5e499caa22753fead639d4fde47c185b6704fbcfeb8805d4f554585bc533620300000800000056030000f90353822080b9034df9034a01828005b9010000000000000020100000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000400000000000000000000008000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000020000000000000000000000000000000002000000000000000000000000000000000f90240f9023d94e9e7593081828a222e38e22578d9241d32504013f863a0413055b58d692937cc2a7d80ca019c17e8d01175e58d11f157ae9124078b01d6a00000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000017c4b5ce0605f63732bfd175fece7ac6b4620fd2b901c00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000049490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000470dcdc5e44064909650113a274b3b36aecb6dc7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000245f8b189ec4c8a819cf573574750db19baa97e1066db26fa76383c83abc9448220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f737564745f65787472615f646174610000000000000000000000000000000000";
    let correct_spv_proof = hex::decode(correct_spv_proof).unwrap();

    let smt_tree = generate_smt_tree();

    let mut key = [0u8; 32];
    let mut height = [0u8; 16];
    height.copy_from_slice(39u128.to_le_bytes().as_ref());

    key[..16].clone_from_slice(&height);

    let block_hash = smt_tree.get(&key.into());
    let merkle_proof = smt_tree.merkle_proof(vec![key.into()]).unwrap();

    let compiled_merkle_proof = merkle_proof
        .compile(vec![(key.into(), block_hash.unwrap())])
        .unwrap();

    let witness = MintTokenWitness::new_builder()
        .mode(Byte::new(0u8))
        .cell_dep_index_list([0u8].to_vec().into())
        .spv_proof(correct_spv_proof.into())
        .merkle_proof(compiled_merkle_proof.0.into())
        .build();
    WitnessArgs::new_builder()
        .lock(Some(witness.as_bytes()).pack())
        .build()
}

fn generate_light_client_data() -> Bytes {
    let smt_tree = generate_smt_tree();

    let merkle_root = smt_tree.root();

    let data = ETHHeaderCellData::new_builder()
        .merkle_root(basic::Byte32::from_slice(merkle_root.as_slice()).unwrap())
        .latest_height(100u64.into())
        .build();

    data.as_bytes()
}

fn generate_lock_script(
    contract_address: &str,
    token_address: &str,
    light_client_typescript_hash: &[u8; 32],
) -> Script {
    let contract_address = ETHAddress::try_from(hex::decode(contract_address).unwrap())
        .unwrap()
        .get_address();
    let token_address = ETHAddress::try_from(hex::decode(token_address).unwrap())
        .unwrap()
        .get_address();

    let lock_args = ETHBridgeLockArgs::new_builder()
        .light_client_typescript_hash(
            basic::Byte32::from_slice(light_client_typescript_hash).unwrap(),
        )
        .eth_contract_address(contract_address.into())
        .eth_token_address(token_address.into())
        .build();
    Script::new_builder()
        .args(lock_args.as_bytes().pack())
        .build()
}

fn generate_sudt_cell() -> (CellOutput, Vec<u8>) {
    let correct_input_lock_hash = [2u8; 32];
    let correct_sudt_script = Script::new_builder()
        .code_hash(packed::Byte32::from_slice(SUDT_CODE_HASH.as_ref()).unwrap())
        .hash_type(SUDT_HASH_TYPE.into())
        .args(Bytes::from(correct_input_lock_hash.to_vec()).pack())
        .build();
    let recipient_lockscript = Script::from_slice(&[
        73u8, 0, 0, 0, 16, 0, 0, 0, 48, 0, 0, 0, 49, 0, 0, 0, 155, 215, 224, 111, 62, 207, 75, 224,
        242, 252, 210, 24, 139, 35, 241, 185, 252, 200, 142, 93, 75, 101, 168, 99, 123, 23, 114,
        59, 189, 163, 204, 232, 1, 20, 0, 0, 0, 71, 13, 205, 197, 228, 64, 100, 144, 150, 80, 17,
        58, 39, 75, 59, 54, 174, 203, 109, 199,
    ])
    .unwrap();
    let cell = CellOutput::new_builder()
        .lock(recipient_lockscript)
        .type_(Some(correct_sudt_script).pack())
        .build();

    let correct_amount: u128 = 100;
    let correct_sudt_extra = "sudt_extra_data".to_string();
    let mut output_data = correct_amount.to_le_bytes().to_vec();
    output_data.extend(correct_sudt_extra.as_bytes().to_vec());
    (cell, output_data)
}

#[test]
fn test_mint_mode_correct() {
    let bridge_lockscript = generate_lock_script(
        "E9e7593081828a222E38E22578D9241D32504013",
        "0000000000000000000000000000000000000000",
        &[1u8; 32],
    );

    let mock = generate_mint_mode_mock(bridge_lockscript);
    let adapter = crate::adapter::ChainAdapter { chain: mock };

    _verify(adapter);
}

#[test]
#[should_panic(expected = "eth spv proof is invalid")]
fn test_mint_mode_invalid_proof() {
    let mut mock = MockAdapter::new();
    let witness = MintTokenWitness::new_builder()
        .cell_dep_index_list([0u8].to_vec().into())
        .build();
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
