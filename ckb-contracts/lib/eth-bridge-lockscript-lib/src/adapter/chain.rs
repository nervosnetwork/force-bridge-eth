use super::{Adapter, BridgeCellDataTuple, ComplexData};
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_tx_hash, QueryIter};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError> {
        load_tx_hash()
    }

    fn load_input_output_data(&self) -> Result<BridgeCellDataTuple, SysError> {
        fn load_data_from_source(source: Source) -> Result<Option<Vec<u8>>, SysError> {
            let data_list = QueryIter::new(load_cell_data, source).collect::<Vec<Vec<u8>>>();
            match data_list.len() {
                0 => Ok(None),
                1 => Ok(Some(data_list[0].clone())),
                _ => Err(SysError::Unknown(100)),
            }
        }
        let tuple = BridgeCellDataTuple(
            load_data_from_source(Source::GroupInput)?,
            load_data_from_source(Source::GroupOutput)?,
        );
        Ok(tuple)
    }

    fn get_complex_data(&self) -> Result<ComplexData, SysError> {
        unimplemented!()
    }
}
