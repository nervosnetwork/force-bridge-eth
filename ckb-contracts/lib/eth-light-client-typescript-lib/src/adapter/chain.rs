#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use super::Adapter;
use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use ckb_std::high_level::{load_cell_data, load_tx_hash, load_witness_args, QueryIter};

use force_eth_types::eth_header_cell::ETHHeaderCellDataView;
use molecule::bytes::Bytes;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_data_from_source(&self, source: Source) -> Option<ETHHeaderCellDataView> {
        let data_list = QueryIter::new(load_cell_data, source).collect::<Vec<Vec<u8>>>();
        match data_list.len() {
            0 => None,
            1 => Some(ETHHeaderCellDataView::from_slice(data_list[0].as_slice())),
            _ => panic!("more than 1 ETH header cell"),
        }
    }

    fn load_data_from_dep(&self, index: usize) -> Vec<u8> {
        load_cell_data(index, Source::CellDep).expect("load data from dep fail")
    }

    fn load_witness_args(&self) -> Bytes {
        load_witness_args(0, Source::Input)
            .expect("load witness fail")
            .input_type()
            .to_opt()
            .expect("witness is none")
            .raw_data()
    }
}
