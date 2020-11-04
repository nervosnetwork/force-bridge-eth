use super::{Adapter, BridgeCellDataTuple};
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{
    load_cell_data, load_cell_lock_hash, load_script_hash, load_witness_args, QueryIter,
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use molecule::bytes::Bytes;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_input_output_data(&self) -> Result<BridgeCellDataTuple, SysError> {
        fn load_data_from_input() -> Option<Vec<u8>> {
            let data_list =
                QueryIter::new(load_cell_data, Source::GroupInput).collect::<Vec<Vec<u8>>>();
            match data_list.len() {
                1 => Some(data_list[0].clone()),
                _ => panic!("inputs have more than 1 bridge cell"),
            }
        }

        fn load_data_from_output() -> Result<Option<Vec<u8>>, SysError> {
            let script_hash = load_script_hash()?;
            let mut output_index = 0;
            let mut output_num = 0;
            let mut data = None;

            loop {
                let cell_lock_hash = load_cell_lock_hash(output_index, Source::Output);
                match cell_lock_hash {
                    Err(SysError::IndexOutOfBound) => break,
                    Err(_err) => panic!("iter output return an error"),
                    Ok(cell_lock_hash) => {
                        if cell_lock_hash == script_hash {
                            data = Some(load_cell_data(output_index, Source::Output)?);
                            output_num += 1;
                            if output_num > 1 {
                                panic!("outputs have more than 1 bridge cell")
                            }
                        }
                        output_index += 1;
                    }
                }
            }
            Ok(data)
        }

        let tuple = BridgeCellDataTuple(load_data_from_input(), load_data_from_output()?);
        Ok(tuple)
    }

    fn load_input_witness_args(&self) -> Result<Bytes, SysError> {
        let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
        if witness_args.is_none() {
            panic!("witness is none");
        }
        Ok(witness_args.to_opt().unwrap().raw_data())
    }

    fn load_cell_dep_data(&self, index: usize) -> Result<Vec<u8>, SysError> {
        load_cell_data(index, Source::CellDep)
    }

    fn check_inputs_lock_hash(&self, data: &[u8]) -> bool {
        QueryIter::new(load_cell_lock_hash, Source::Input)
            .filter(|hash| hash.as_ref() == data)
            .count()
            > 0
    }
}
