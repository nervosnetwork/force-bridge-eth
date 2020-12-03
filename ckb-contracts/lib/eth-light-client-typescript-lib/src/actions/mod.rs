mod helper;

#[cfg(not(feature = "std"))]
use alloc::vec;

use crate::adapter::Adapter;
use crate::debug;

use ckb_std::ckb_constants::Source;
use eth_spv_lib::eth_types::*;
use force_eth_types::{
    config::CONFIRM,
    eth_header_cell::ETHHeaderCellDataView,
    generated::{
        basic::BytesVecReader,
        eth_header_cell::{ETHChainReader, ETHHeaderInfoReader, ETHLightClientWitnessReader},
    },
};
use molecule::prelude::Reader;

pub const MAIN_HEADER_CACHE_LIMIT: usize = 500;
pub const UNCLE_HEADER_CACHE_LIMIT: usize = 10;

pub fn verify_add_headers<T: Adapter>(data_loader: T) {
    debug!("start add headers");
    let input_data = data_loader.load_data_from_source(Source::GroupInput);
    let output_data = data_loader
        .load_data_from_source(Source::GroupOutput)
        .expect("output should not be none");

    let witness_args = data_loader.load_witness_args();
    if ETHLightClientWitnessReader::verify(&witness_args, false).is_err() {
        panic!("get_witness_reader, invalid witness");
    }
    let witness_reader = ETHLightClientWitnessReader::new_unchecked(&witness_args);

    let new_headers_len = witness_reader.headers().len();
    let mut new_headers_rlp = vec![];
    let mut new_headers = vec![];
    for i in 0..new_headers_len {
        let header_rlp = witness_reader.headers().get_unchecked(i).raw_data();
        new_headers_rlp.push(header_rlp);

        let header: BlockHeader = rlp::decode(header_rlp).unwrap();
        new_headers.push(header);
    }
    verify_witness_headers_consequent(&new_headers);

    match input_data {
        Some(data) => verify_push_header(&data, &output_data, &new_headers, &new_headers_rlp),
        None => {
            assert_eq!(
                data_loader.load_first_outpoint(),
                data_loader.load_script_args(),
                "invalid first cell id"
            );
            verify_init_header(&output_data, &new_headers, &new_headers_rlp)
        }
    };

    //todo verify merkle proof
}

fn verify_witness_headers_consequent(headers: &[BlockHeader]) {
    let new_headers_len = headers.len();
    for i in 1..new_headers_len {
        if headers[i - 1].hash.unwrap() != headers[i].parent_hash {
            panic!("witness headers are not consequent");
        }
    }
}

fn verify_init_header(
    output: &ETHHeaderCellDataView,
    new_headers: &[BlockHeader],
    new_headers_rlp: &[&[u8]],
) {
    debug!("init header list.");
    if ETHChainReader::verify(&output.headers, false).is_err() {
        panic!("verify_init_header, invalid output headers");
    }
    let chain_reader = ETHChainReader::new_unchecked(&output.headers);
    let main_reader = chain_reader.main();
    let uncle_reader = chain_reader.uncle();
    assert_eq!(
        main_reader.len() == new_headers.len(),
        true,
        "invalid main chain"
    );
    assert_eq!(uncle_reader.is_empty(), true, "invalid uncle chain");

    check_new_headers_in_output(new_headers, new_headers_rlp, main_reader);
}

fn verify_push_header(
    input: &ETHHeaderCellDataView,
    output: &ETHHeaderCellDataView,
    new_headers: &[BlockHeader],
    new_headers_rlp: &[&[u8]],
) {
    let (main_input_reader, uncle_input_reader) = get_main_and_uncle_from_headers(&input.headers);
    let (main_output_reader, uncle_output_reader) =
        get_main_and_uncle_from_headers(&output.headers);

    let (_, main_tail_header_output) = get_tail_info(main_output_reader);

    // header is on uncle chain, just do append.
    if main_tail_header_output != new_headers_rlp[new_headers_rlp.len() - 1] {
        check_new_headers_in_output(new_headers, new_headers_rlp, uncle_output_reader);
        return verify_uncle_header(
            new_headers,
            main_input_reader,
            main_output_reader,
            uncle_input_reader,
            uncle_output_reader,
        );
    }

    check_new_headers_in_output(new_headers, new_headers_rlp, main_output_reader);

    verify_main_header(
        new_headers,
        main_input_reader,
        main_output_reader,
        uncle_input_reader,
        uncle_output_reader,
    )
}

fn get_main_and_uncle_from_headers(headers: &[u8]) -> (BytesVecReader, BytesVecReader) {
    if ETHChainReader::verify(headers, false).is_err() {
        panic!("get_main_and_uncle_from_headers, invalid headers");
    }
    let chain_reader = ETHChainReader::new_unchecked(headers);
    let main_reader = chain_reader.main();
    let uncle_reader = chain_reader.uncle();
    (main_reader, uncle_reader)
}

fn get_tail_info(main_reader: BytesVecReader) -> (ETHHeaderInfoReader, &[u8]) {
    let main_tail_info = main_reader.get_unchecked(main_reader.len() - 1).raw_data();
    if ETHHeaderInfoReader::verify(&main_tail_info, false).is_err() {
        panic!("get_tail_info, invalid main tail info");
    }
    let main_tail_info_reader = ETHHeaderInfoReader::new_unchecked(main_tail_info);
    let main_tail_header = main_tail_info_reader.header().raw_data();

    (main_tail_info_reader, main_tail_header)
}

fn check_new_headers_in_output(
    headers: &[BlockHeader],
    headers_rlp: &[&[u8]],
    output_reader: BytesVecReader,
) {
    let output_len = output_reader.len();
    let new_headers_len = headers.len();
    let start_index = output_len - new_headers_len;

    for i in start_index..output_len {
        let header_info = output_reader.get_unchecked(i).raw_data();
        if ETHHeaderInfoReader::verify(&header_info, false).is_err() {
            panic!("check_new_headers_in_output, invalid header info");
        }
        let header_info_reader = ETHHeaderInfoReader::new_unchecked(header_info);
        assert_eq!(
            headers[i - start_index].hash.unwrap().0.as_bytes(),
            header_info_reader.hash().raw_data(),
            "new header is not in outputs"
        );
        assert_eq!(
            headers_rlp[i - start_index],
            header_info_reader.header().raw_data(),
            "invalid header raw data",
        );
    }
}

fn verify_uncle_header(
    headers: &[BlockHeader],
    main_input_reader: BytesVecReader,
    main_output_reader: BytesVecReader,
    uncle_input_reader: BytesVecReader,
    uncle_output_reader: BytesVecReader,
) {
    debug!("warning: the new header is not on main chain.");
    verify_uncle_original_chain_data(headers.len(), uncle_input_reader, uncle_output_reader);
    // the main chain should be the same.
    assert_eq!(main_output_reader.as_slice(), main_input_reader.as_slice());
}

fn verify_main_header(
    headers: &[BlockHeader],
    main_input_reader: BytesVecReader,
    main_output_reader: BytesVecReader,
    uncle_input_reader: BytesVecReader,
    uncle_output_reader: BytesVecReader,
) {
    debug!("the new header is on main chain");

    verify_main_and_uncle_length(
        main_input_reader,
        main_output_reader,
        uncle_input_reader,
        uncle_output_reader,
    );

    let (main_tail_info_input_reader, main_tail_header_input) = get_tail_info(main_input_reader);
    let (main_tail_info_output_reader, _) = get_tail_info(main_output_reader);

    let main_tail_input: BlockHeader =
        rlp::decode(main_tail_header_input.to_vec().as_slice()).unwrap();
    debug!(
        "new header parent hash: {:?}, input main chain tail hash: {:?}",
        headers[0].parent_hash.0,
        main_tail_input.hash.unwrap().0
    );

    // if header.parent_hash == tail_input.hash => the chain is not reorg.
    // else do reorg.
    if main_tail_input.hash.unwrap() == headers[0].parent_hash {
        verify_main_not_reorg(
            headers,
            main_tail_info_input_reader,
            main_tail_info_output_reader,
        );
        verify_original_chain_data(headers.len(), main_input_reader, main_output_reader);
        // the uncle chain should be the same.
        assert_eq!(
            uncle_input_reader.as_slice(),
            uncle_output_reader.as_slice()
        );
    } else {
        debug!("warning: the main chain had been reorged.");
        verify_difficulty(
            headers,
            main_tail_info_input_reader,
            main_tail_info_output_reader,
            main_input_reader,
            uncle_input_reader,
        );

        // header.number < main_tail_input.number
        // assert_eq!(main_tail_input.number - header.number > 0, true)
        let mut number = headers[0].clone().number - 1;
        let mut current_hash = headers[0].clone().parent_hash;
        loop {
            if number == 0 {
                panic!("number should be bigger than 0");
            }
            // find parent header.
            if !check_parent_hash_on_main(
                main_input_reader,
                main_tail_input.clone(),
                current_hash,
                number,
            ) {
                // the parent header is on uncle chain.
                debug!("the parent header is on uncle chain");
                traverse_uncle_chain(uncle_input_reader, &mut current_hash, &mut number);
            } else {
                let offset = (main_tail_input.number - number) as usize;
                // the parent header is on main chain.
                // make sure the main chain is right.
                let mut input_data = vec![];
                for i in 0..main_input_reader.len() - offset {
                    input_data.push(main_input_reader.get_unchecked(i).raw_data())
                }
                let mut output_data = vec![];
                for i in 0..main_output_reader.len() - headers.len() {
                    output_data.push(main_output_reader.get_unchecked(i).raw_data())
                }
                assert_eq!(input_data, output_data);
                // FIXME: make sure the uncle chain is right.
                if uncle_input_reader.len() + offset > UNCLE_HEADER_CACHE_LIMIT {
                    verify_uncle_over_cache_limit(
                        main_input_reader,
                        uncle_input_reader,
                        uncle_output_reader,
                        offset,
                    );
                }
                break;
            }
        }
    }
}

fn verify_main_and_uncle_length(
    main_input_reader: BytesVecReader,
    main_output_reader: BytesVecReader,
    uncle_input_reader: BytesVecReader,
    uncle_output_reader: BytesVecReader,
) {
    if main_output_reader.len() > MAIN_HEADER_CACHE_LIMIT
        || main_input_reader.len() > MAIN_HEADER_CACHE_LIMIT
        || uncle_output_reader.len() > UNCLE_HEADER_CACHE_LIMIT
        || uncle_input_reader.len() > UNCLE_HEADER_CACHE_LIMIT
    {
        panic!("main or uncle len exceed max");
    }
}

fn verify_main_not_reorg(
    headers: &[BlockHeader],
    main_tail_info_input_reader: ETHHeaderInfoReader,
    main_tail_info_output_reader: ETHHeaderInfoReader,
) {
    debug!("the main chain is not reorg.");
    let prev_difficult = main_tail_info_input_reader.total_difficulty().raw_data();
    let left = main_tail_info_output_reader.total_difficulty().raw_data();

    let mut header_difficulty = 0;
    for header in headers {
        header_difficulty += header.difficulty.0.as_u64();
    }

    let right: u64 = header_difficulty;
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
}

fn verify_difficulty(
    headers: &[BlockHeader],
    main_tail_info_input_reader: ETHHeaderInfoReader,
    main_tail_info_output_reader: ETHHeaderInfoReader,
    main_input_reader: BytesVecReader,
    uncle_input_reader: BytesVecReader,
) {
    let input_total_difficulty = to_u64(main_tail_info_input_reader.total_difficulty().raw_data());
    let output_total_difficulty =
        to_u64(main_tail_info_output_reader.total_difficulty().raw_data());

    let mut headers_difficulty = 0;
    for header in headers {
        headers_difficulty += header.difficulty.0.as_u64();
    }

    let (_, header_parent_difficulty) =
        get_parent_header(headers[0].clone(), main_input_reader, uncle_input_reader);

    //difficulty need verify! output_total_difficulty == header.difficulty + header.parent.total_difficulty
    assert_eq!(
        output_total_difficulty,
        headers_difficulty + header_parent_difficulty,
        "invalid difficulty."
    );

    if output_total_difficulty < input_total_difficulty {
        panic!("output difficulty less than input difficulty")
    }
}

fn verify_uncle_over_cache_limit(
    main_input_reader: BytesVecReader,
    uncle_input_reader: BytesVecReader,
    uncle_output_reader: BytesVecReader,
    offset: usize,
) {
    let mut uncle_input_data = vec![];
    let begin = uncle_input_reader.len() + offset - UNCLE_HEADER_CACHE_LIMIT;

    for i in begin..uncle_input_reader.len() {
        uncle_input_data.push(uncle_input_reader.get_unchecked(i).raw_data())
    }
    for i in main_input_reader.len() - offset..main_input_reader.len() {
        uncle_input_data.push(main_input_reader.get_unchecked(i).raw_data())
    }
    let mut uncle_output_data = vec![];
    for i in 0..uncle_output_reader.len() {
        uncle_output_data.push(uncle_output_reader.get_unchecked(i).raw_data())
    }
    assert_eq!(
        uncle_input_data, uncle_output_data,
        "invalid uncle chain data"
    );
}

fn check_parent_hash_on_main(
    main_input_reader: BytesVecReader,
    main_tail_input: BlockHeader,
    current_hash: H256,
    number: u64,
) -> bool {
    if main_tail_input.number <= number {
        return false;
    }
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
        return true;
    }
    false
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

fn verify_uncle_original_chain_data(
    new_headers_len: usize,
    input_reader: BytesVecReader,
    output_reader: BytesVecReader,
) {
    let input_len = input_reader.len();
    let output_len = output_reader.len();

    if input_len == output_len && output_len == UNCLE_HEADER_CACHE_LIMIT {
        let mut input_data = vec![];
        for i in new_headers_len..input_len {
            input_data.push(input_reader.get_unchecked(i).raw_data())
        }

        let mut output_data = vec![];
        for i in 0..output_len - new_headers_len {
            output_data.push(output_reader.get_unchecked(i).raw_data())
        }

        assert_eq!(input_data, output_data, "invalid output data.");
    } else if input_len < output_len {
        if output_len <= UNCLE_HEADER_CACHE_LIMIT {
            let mut input_data = vec![];
            for i in 0..input_len {
                input_data.push(input_reader.get_unchecked(i).raw_data())
            }
            let mut output_data = vec![];
            for i in 0..output_len - new_headers_len {
                output_data.push(output_reader.get_unchecked(i).raw_data())
            }
            assert_eq!(input_data, output_data, "invalid output data.");
        } else {
            panic!("uncle data exceed cache limit")
        }
    } else {
        panic!("input uncle length over output uncle")
    }
}

fn verify_original_chain_data(
    new_headers_len: usize,
    input_reader: BytesVecReader,
    output_reader: BytesVecReader,
) {
    let input_len = input_reader.len();
    let output_len = output_reader.len();

    if input_reader.len() == output_len && output_len == MAIN_HEADER_CACHE_LIMIT {
        let mut input_data = vec![];
        for i in new_headers_len..input_len {
            input_data.push(input_reader.get_unchecked(i).raw_data())
        }

        let mut output_data = vec![];
        for i in 0..output_len - new_headers_len {
            output_data.push(output_reader.get_unchecked(i).raw_data())
        }

        assert_eq!(input_data, output_data, "invalid output data.");
    } else if input_len < output_len {
        if output_len <= CONFIRM {
            let mut input_data = vec![];
            for i in 0..input_len {
                input_data.push(input_reader.get_unchecked(i).raw_data())
            }
            let mut output_data = vec![];
            for i in 0..output_len - new_headers_len {
                output_data.push(output_reader.get_unchecked(i).raw_data())
            }
            assert_eq!(input_data, output_data, "invalid output data.");
        } else {
            let mut input_data = vec![];
            for i in 0..input_len {
                input_data.push(input_reader.get_unchecked(i).raw_data())
            }

            let mut start_index = 0;
            if input_len > CONFIRM {
                start_index = input_len - CONFIRM;
            }
            #[allow(clippy::needless_range_loop)]
            for i in start_index..output_len - CONFIRM {
                if ETHHeaderInfoReader::verify(input_data[i], false).is_err() {
                    panic!("invalid header info");
                }
                let header_info_reader = ETHHeaderInfoReader::new_unchecked(input_data[i]);
                let hash = header_info_reader.hash().raw_data();
                input_data[i] = hash;
            }

            let mut output_data = vec![];
            for i in 0..input_len {
                output_data.push(output_reader.get_unchecked(i).raw_data());
            }
            assert_eq!(input_data, output_data, "invalid output data.");
        }
    } else {
        panic!("invalid data")
    }
}

fn to_u64(data: &[u8]) -> u64 {
    let mut res = [0u8; 8];
    res.copy_from_slice(data);
    u64::from_le_bytes(res)
}
