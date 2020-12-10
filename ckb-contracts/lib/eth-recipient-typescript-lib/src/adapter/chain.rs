use super::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::packed::Script;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_cell_type, QueryIter};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_output_data_by_trait(&self) -> Vec<Vec<u8>> {
        QueryIter::new(load_cell_data, Source::GroupOutput).collect::<Vec<Vec<u8>>>()
    }

    fn load_cell_data_by_trait(&self, index: usize, source: Source) -> Vec<u8> {
        load_cell_data(index, source).expect("laod cell data fail")
    }

    fn load_cell_type_by_trait(
        &self,
        index: usize,
        source: Source,
    ) -> Result<Option<Script>, SysError> {
        load_cell_type(index, source)
    }
}
