mod helper;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use crate::adapter::Adapter;
use crate::debug;

use ckb_std::ckb_constants::Source;
use eth_spv_lib::eth_types::*;
use force_eth_types::{
    eth_header_cell::ETHHeaderCellDataView,
    generated::{
        basic::BytesVecReader,
        eth_header_cell::{
            DagsMerkleRootsReader, DoubleNodeWithMerkleProofReader, ETHChainReader,
            ETHHeaderInfoReader, ETHLightClientWitnessReader,
        },
    },
};
use helper::{DoubleNodeWithMerkleProof, *};
use molecule::prelude::Reader;

pub const MAIN_HEADER_CACHE_LIMIT: usize = 500;
pub const UNCLE_HEADER_CACHE_LIMIT: usize = 10;
pub const CONFIRM: usize = 10;

pub fn verify_add_headers<T: Adapter>(data_loader: T) {
    let input_data = data_loader.load_data_from_source(Source::GroupInput);
    let output_data = data_loader
        .load_data_from_source(Source::GroupOutput)
        .expect("output should not be none");

    verify_witness(data_loader, &input_data, &output_data);
}

/// ensure the new header is valid.
fn verify_witness<T: Adapter>(
    data_loader: T,
    input: &Option<ETHHeaderCellDataView>,
    output: &ETHHeaderCellDataView,
) {
    debug!("verify verify_witness data.");
    let witness_args = data_loader.load_witness_args();
    if ETHLightClientWitnessReader::verify(&witness_args, false).is_err() {
        panic!("invalid witness");
    }
    let witness = ETHLightClientWitnessReader::new_unchecked(&witness_args);
    // parse header
    let header_raw = witness.header().raw_data();
    // check input && output data
    let (header, prev) = match input {
        Some(data) => verify_input_output_data(data, output, header_raw),
        None => init_header_info(output, header_raw),
    };
    // parse merkle proof
    let mut proofs = vec![];
    for i in 0..witness.merkle_proof().len() {
        let proof_raw = witness.merkle_proof().get_unchecked(i).raw_data();
        let proof = parse_proof(proof_raw);
        proofs.push(proof);
    }
    // parse dep data
    let merkle_root = parse_dep_data(data_loader, witness, header.number);
    if !verify_header(&header, prev, merkle_root, &proofs) {
        panic!("verify header fail");
    }
}

fn init_header_info(
    output: &ETHHeaderCellDataView,
    header_raw: &[u8],
) -> (BlockHeader, Option<BlockHeader>) {
    debug!("init header list.");
    let header: BlockHeader = rlp::decode(header_raw.to_vec().as_slice()).unwrap();
    if ETHChainReader::verify(&output.headers, false).is_err() {
        panic!("invalid output headers");
    }
    let chain_reader = ETHChainReader::new_unchecked(&output.headers);
    let main_reader = chain_reader.main();
    let uncle_reader = chain_reader.uncle();
    assert_eq!(main_reader.len() == 1, true, "invalid main chain");
    assert_eq!(uncle_reader.is_empty(), true, "invalid uncle chain");
    let main_tail_info = main_reader.get_unchecked(0).raw_data();
    if ETHHeaderInfoReader::verify(&main_tail_info, false).is_err() {
        panic!("invalid main tail info");
    }
    let main_tail_reader = ETHHeaderInfoReader::new_unchecked(main_tail_info);
    let main_tail_raw = main_tail_reader.header().raw_data();
    let total_difficulty = main_tail_reader.total_difficulty().raw_data();
    let difficulty: u64 = header.difficulty.0.as_u64();
    let hash = main_tail_reader.hash().raw_data();
    assert_eq!(main_tail_raw, header_raw, "invalid header raw data");
    assert_eq!(
        to_u64(&total_difficulty),
        difficulty,
        "invalid total difficulty"
    );
    assert_eq!(hash, header.hash.unwrap().0.as_bytes(), "invalid hash");
    (header, Option::None)
}

fn verify_input_output_data(
    input: &ETHHeaderCellDataView,
    output: &ETHHeaderCellDataView,
    header_raw: &[u8],
) -> (BlockHeader, Option<BlockHeader>) {
    debug!("verify input && output data. make sure the main chain is right.");
    let header: BlockHeader =
        rlp::decode(header_raw.to_vec().as_slice()).expect("rlp decode header_raw fail");
    debug!("header after decode is {:?}", header);

    if ETHChainReader::verify(&input.headers, false).is_err() {
        panic!("invalid input headers");
    }
    let chain_input_reader = ETHChainReader::new_unchecked(&input.headers);
    let main_input_reader = chain_input_reader.main();
    debug!(
        "input: the main chain length: {:?}",
        main_input_reader.len()
    );
    let uncle_input_reader = chain_input_reader.uncle();
    if ETHChainReader::verify(&output.headers, false).is_err() {
        panic!("invalid output headers");
    }
    let chain_output_reader = ETHChainReader::new_unchecked(&output.headers);
    let main_output_reader = chain_output_reader.main();
    let uncle_output_reader = chain_output_reader.uncle();
    debug!(
        "output: the main chain length: {:?}",
        main_output_reader.len()
    );
    // header is on main chain.
    let main_tail_info_input = main_input_reader
        .get_unchecked(main_input_reader.len() - 1)
        .raw_data();
    if ETHHeaderInfoReader::verify(&main_tail_info_input, false).is_err() {
        panic!("invalid main tail info input");
    }
    let main_tail_info_input_reader = ETHHeaderInfoReader::new_unchecked(main_tail_info_input);
    let main_tail_header_input = main_tail_info_input_reader.header().raw_data();

    let main_tail_info_output = main_output_reader
        .get_unchecked(main_output_reader.len() - 1)
        .raw_data();
    if ETHHeaderInfoReader::verify(&main_tail_info_output, false).is_err() {
        panic!("invalid main tail info output");
    }
    let main_tail_info_output_reader = ETHHeaderInfoReader::new_unchecked(main_tail_info_output);
    let main_tail_header_output = main_tail_info_output_reader.header().raw_data();
    let mut prev: Option<BlockHeader> = Option::None;
    // header is on main chain.
    if main_tail_header_output == header_raw {
        debug!("the new header is on main chain");
        assert_eq!(
            main_tail_info_output_reader.hash().raw_data(),
            header.hash.unwrap().0.as_bytes()
        );
        let main_tail_input: BlockHeader =
            rlp::decode(main_tail_header_input.to_vec().as_slice()).unwrap();
        debug!("new header parent hash: {:?} ", header.parent_hash.0);
        debug!(
            "input main chain tail hash: {:?}",
            main_tail_input.hash.unwrap().0
        );
        if main_output_reader.len() > 1 {
            let header_raw = main_output_reader
                .get_unchecked(main_output_reader.len() - 2)
                .raw_data();
            prev = Option::Some(extra_header(header_raw));
        }

        if main_output_reader.len() > MAIN_HEADER_CACHE_LIMIT
            || main_input_reader.len() > MAIN_HEADER_CACHE_LIMIT
            || uncle_output_reader.len() > UNCLE_HEADER_CACHE_LIMIT
            || uncle_input_reader.len() > UNCLE_HEADER_CACHE_LIMIT
        {
            panic!("main or uncle len exceed max");
        }
        // if header.parent_hash == tail_input.hash => the chain is not reorg.
        // else do reorg.
        if main_tail_input.hash.unwrap() == header.parent_hash {
            debug!("the main chain is not reorg.");
            let prev_difficult = main_tail_info_input_reader.total_difficulty().raw_data();
            let left = main_tail_info_output_reader.total_difficulty().raw_data();
            let right: u64 = header.difficulty.0.as_u64();
            debug!("The total difficulty of the output chain is the total difficulty of the input chain plus the difficulty of the new block");
            debug!(
                "left difficulty u64: {} right difficulty u64: {}",
                to_u64(&left),
                right.checked_add(to_u64(&prev_difficult)).unwrap()
            );
            assert_eq!(
                to_u64(&left),
                right.checked_add(to_u64(&prev_difficult)).unwrap(),
                "invalid difficulty."
            );

            debug!("the uncle chain should be the same");
            verify_original_chain_data(
                main_input_reader,
                main_output_reader,
                MAIN_HEADER_CACHE_LIMIT,
            );
            // the uncle chain should be the same.
            assert_eq!(
                uncle_input_reader.as_slice(),
                uncle_output_reader.as_slice()
            );
        } else {
            debug!("warning: the main chain had been reorged.");
            let left = main_tail_info_input_reader.total_difficulty().raw_data();
            let right = main_tail_info_output_reader.total_difficulty().raw_data();
            //difficulty need verify! right == header.difficulty + header.parent.total_difficulty
            let (_, difficulty) =
                get_parent_header(header.clone(), main_input_reader, uncle_input_reader);
            assert_eq!(
                to_u64(right),
                header.difficulty.0.as_u64() + difficulty,
                "invalid difficulty."
            );

            if to_u64(&right) >= to_u64(&left) {
                // header.number < main_tail_input.number
                // assert_eq!(main_tail_input.number - header.number > 0, true)
                let mut number = header.number - 1;
                let mut current_hash = header.parent_hash;
                loop {
                    if number == 0 {
                        panic!("invalid data");
                    }
                    // find parent header.
                    if main_tail_input.number <= number {
                        // the parent header is on uncle chain.
                        debug!("the parent header is on uncle chain");
                        traverse_uncle_chain(uncle_input_reader, &mut current_hash, &mut number);
                    } else {
                        let offset = (main_tail_input.number - number) as usize;
                        debug!("offset: {:?}", offset);
                        assert_eq!(offset < main_input_reader.len(), true, "invalid cell data");
                        assert_eq!(offset < CONFIRM, true, "can not revert confirmed block.");
                        let header_info_temp = main_input_reader
                            .get_unchecked(main_input_reader.len() - 1 - offset)
                            .raw_data();
                        let hash_temp = extra_hash(header_info_temp);
                        debug!(
                            "hash_temp: {:?} current_hash: {:?}",
                            hash_temp,
                            current_hash.0.as_bytes()
                        );
                        if hash_temp == current_hash.0.as_bytes() {
                            // the parent header is on main chain.
                            // make sure the main chain is right.
                            let mut input_data = vec![];
                            for i in 0..main_input_reader.len() - offset {
                                input_data.push(main_input_reader.get_unchecked(i).raw_data())
                            }
                            let mut output_data = vec![];
                            for i in 0..main_output_reader.len() - 1 {
                                output_data.push(main_output_reader.get_unchecked(i).raw_data())
                            }
                            assert_eq!(input_data, output_data);
                            // FIXME: make sure the uncle chain is right.
                            if uncle_input_reader.len() + offset > UNCLE_HEADER_CACHE_LIMIT {
                                let mut uncle_input_data = vec![];
                                let begin =
                                    uncle_input_reader.len() + offset - UNCLE_HEADER_CACHE_LIMIT;

                                for i in begin..uncle_input_reader.len() {
                                    uncle_input_data
                                        .push(uncle_input_reader.get_unchecked(i).raw_data())
                                }
                                for i in main_input_reader.len() - offset..main_input_reader.len() {
                                    uncle_input_data
                                        .push(main_input_reader.get_unchecked(i).raw_data())
                                }
                                let mut uncle_output_data = vec![];
                                for i in 0..uncle_output_reader.len() {
                                    uncle_output_data
                                        .push(uncle_output_reader.get_unchecked(i).raw_data())
                                }
                                assert_eq!(
                                    uncle_input_data, uncle_output_data,
                                    "invalid uncle chain data"
                                );
                            }
                            break;
                        } else {
                            // the parent header is on uncle chain.
                            traverse_uncle_chain(
                                uncle_input_reader,
                                &mut current_hash,
                                &mut number,
                            );
                        }
                    }
                }
            } else {
                panic!("invalid data");
            }
        }
    } else {
        debug!("warning: the new header is not on main chain.");
        // the header is on uncle chain. just do append.
        verify_original_chain_data(
            uncle_input_reader,
            uncle_output_reader,
            UNCLE_HEADER_CACHE_LIMIT,
        );
        // the main chain should be the same.
        assert_eq!(main_output_reader.as_slice(), main_input_reader.as_slice());
        let (_prev, _) = get_parent_header(header.clone(), main_input_reader, uncle_input_reader);
        prev = Option::Some(_prev);
    }
    // assert_eq!(main_output_reader.get_unchecked(main_output_reader.len() - 1).raw_data(), header_raw);
    (header, prev)
}

fn extra_header(header_info_raw: &[u8]) -> BlockHeader {
    if ETHHeaderInfoReader::verify(&header_info_raw, false).is_err() {
        panic!("invalid header info raw");
    }
    let reader = ETHHeaderInfoReader::new_unchecked(header_info_raw);
    let header_raw = reader.header().raw_data();
    rlp::decode(header_raw.to_vec().as_slice()).unwrap()
}

fn extra_difficulty(header_info_raw: &[u8]) -> u64 {
    if ETHHeaderInfoReader::verify(&header_info_raw, false).is_err() {
        panic!("invalid header info raw");
    }
    let reader = ETHHeaderInfoReader::new_unchecked(header_info_raw);
    let total_difficulty = reader.total_difficulty().raw_data();
    to_u64(total_difficulty)
}

fn extra_hash(header_info_raw: &[u8]) -> &[u8] {
    if ETHHeaderInfoReader::verify(&header_info_raw, false).is_err() {
        panic!("invalid header info raw");
    }
    let reader = ETHHeaderInfoReader::new_unchecked(header_info_raw);
    reader.hash().raw_data()
}

fn get_parent_header(
    header: BlockHeader,
    main_input_reader: BytesVecReader,
    uncle_input_reader: BytesVecReader,
) -> (BlockHeader, u64) {
    let main_tail_info = main_input_reader
        .get_unchecked(main_input_reader.len() - 1)
        .raw_data();
    let main_tail = extra_header(main_tail_info);
    let offset = (main_tail.number - header.number + 1) as usize;
    assert_eq!(offset < CONFIRM, true, "can not revert a confirmed block.");
    let target_raw = main_input_reader
        .get_unchecked(main_input_reader.len() - 1 - offset)
        .raw_data();
    let target = extra_header(target_raw);
    if target.hash.unwrap() == header.parent_hash {
        let difficulty = extra_difficulty(target_raw);
        (target, difficulty)
    } else {
        let mut index = (uncle_input_reader.len() - 1) as isize;
        loop {
            if index < 0 {
                panic!("invalid data");
            }
            let uncle_tail_input = uncle_input_reader.get_unchecked(index as usize).raw_data();
            let uncle_header = extra_header(uncle_tail_input);
            if uncle_header.hash.unwrap() == header.hash.unwrap() {
                let difficulty = extra_difficulty(uncle_tail_input);
                return (uncle_header, difficulty);
            } else {
                index -= 1;
            }
        }
    }
}

fn traverse_uncle_chain(
    uncle_input_reader: BytesVecReader,
    current_hash: &mut H256,
    number: &mut u64,
) {
    debug!("index: {:?}", uncle_input_reader.len());
    let mut index = (uncle_input_reader.len() - 1) as isize;
    loop {
        if index < 0 {
            panic!("invalid data");
        }
        let uncle_tail_input = uncle_input_reader.get_unchecked(index as usize).raw_data();
        let uncle_header = extra_header(uncle_tail_input);
        if uncle_header.hash.unwrap().0.as_bytes() == current_hash.0.as_bytes() {
            // TODO: make sure the header on uncle chain also exist on the main chain.
            *number -= 1;
            *current_hash = uncle_header.parent_hash;
            break;
        } else {
            index -= 1;
        }
    }
}

fn verify_original_chain_data(
    input_reader: BytesVecReader,
    output_reader: BytesVecReader,
    limit: usize,
) {
    if input_reader.len() == output_reader.len() && output_reader.len() == limit {
        let mut input_data = vec![];
        for i in 1..input_reader.len() {
            input_data.push(input_reader.get_unchecked(i).raw_data())
        }
        let mut output_data = vec![];
        for i in 0..output_reader.len() - 1 {
            output_data.push(output_reader.get_unchecked(i).raw_data())
        }
        assert_eq!(input_data, output_data, "invalid output data.");
    } else if input_reader.len() < output_reader.len() {
        let mut input_data = vec![];
        for i in 0..input_reader.len() {
            input_data.push(input_reader.get_unchecked(i).raw_data())
        }
        let mut output_data = vec![];
        for i in 0..output_reader.len() - 1 {
            output_data.push(output_reader.get_unchecked(i).raw_data())
        }
        assert_eq!(input_data, output_data, "invalid output data.");
    } else {
        panic!("invalid data")
    }
}

fn parse_proof(proof_raw: &[u8]) -> DoubleNodeWithMerkleProof {
    if DoubleNodeWithMerkleProofReader::verify(&proof_raw, false).is_err() {
        panic!("invalid proof raw");
    }
    let merkle_proof = DoubleNodeWithMerkleProofReader::new_unchecked(proof_raw);
    let mut dag_nodes = vec![];
    for i in 0..merkle_proof.dag_nodes().len() {
        let mut node = [0u8; 64];
        node.copy_from_slice(merkle_proof.dag_nodes().get_unchecked(i).raw_data());
        dag_nodes.push(H512(node.into()));
    }
    let mut proofs = vec![];
    for i in 0..merkle_proof.proof().len() {
        let mut proof = [0u8; 16];
        proof.copy_from_slice(merkle_proof.proof().get_unchecked(i).raw_data());
        proofs.push(H128(proof.into()));
    }
    DoubleNodeWithMerkleProof::new(dag_nodes, proofs)
}

fn parse_dep_data<T: Adapter>(
    data_loader: T,
    witness: ETHLightClientWitnessReader,
    number: u64,
) -> H128 {
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();
    if cell_dep_index_list.len() != 1 {
        panic!("witness cell dep index len is not 1");
    }
    let dep_data = data_loader.load_data_from_dep(cell_dep_index_list[0].into());
    // debug!("dep data is {:?}", &dep_data);
    if DagsMerkleRootsReader::verify(&dep_data, false).is_err() {
        panic!("invalid dags");
    }
    let dags_reader = DagsMerkleRootsReader::new_unchecked(&dep_data);
    let idx: usize = (number / 30000) as usize;
    let merkle_root_tmp = dags_reader
        .dags_merkle_roots()
        .get_unchecked(idx)
        .raw_data();
    let mut merkle_root = [0u8; 16];
    merkle_root.copy_from_slice(merkle_root_tmp);
    H128(merkle_root.into())
}

fn to_u64(data: &[u8]) -> u64 {
    let mut res = [0u8; 8];
    res.copy_from_slice(data);
    u64::from_le_bytes(res)
}
