#[cfg(feature = "std")]
use mockall::predicate::*;
#[cfg(feature = "std")]
use mockall::*;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::QueryIter;
use contracts_helper::data_loader::DataLoader;

use molecule::bytes::Bytes;
use molecule::prelude::Entity;

use std::prelude::v1::*;

#[cfg_attr(feature = "std", automock)]
pub trait Adapter {
    fn load_data_len_from_source(&self, source: Source) -> usize;
    fn load_script_args(&self) -> Bytes;
    fn load_first_outpoint(&self) -> Bytes;
}
pub struct ChainAdapter<T: DataLoader> {
    pub chain: T,
}

impl<T> Adapter for ChainAdapter<T>
where
    T: DataLoader,
{
    fn load_data_len_from_source(&self, source: Source) -> usize {
        QueryIter::new(
            |index, source| self.chain.load_cell_data(index, source),
            source,
        )
        .count()
    }

    fn load_script_args(&self) -> Bytes {
        self.chain
            .load_script()
            .expect("load script fail")
            .args()
            .raw_data()
    }

    fn load_first_outpoint(&self) -> Bytes {
        self.chain
            .load_input_out_point(0, Source::Input)
            .expect("load input outpoint fail")
            .as_bytes()
    }
}
