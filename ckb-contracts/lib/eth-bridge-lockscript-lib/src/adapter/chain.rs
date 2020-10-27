use super::{Adapter, BridgeCellDataTuple, ComplexData};
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{
    load_cell_data, load_cell_lock_hash, load_script_hash, load_tx_hash, QueryIter,
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError> {
        load_tx_hash()
    }

    fn load_input_output_data(&self) -> Result<BridgeCellDataTuple, SysError> {
        fn load_data_from_input() -> Option<Vec<u8>> {
            let data_list =
                QueryIter::new(load_cell_data, Source::GroupInput).collect::<Vec<Vec<u8>>>();
            match data_list.len() {
                0 => None,
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

    fn get_complex_data(&self) -> Result<ComplexData, SysError> {
        unimplemented!()
    }
}
