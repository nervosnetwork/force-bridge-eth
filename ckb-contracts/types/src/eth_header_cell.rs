use molecule::bytes::Bytes;
use crate::generated::eth_header_cell::ETHHeaderCellDataReader;
use molecule::prelude::{Entity, Reader};

#[derive(Debug)]
pub struct ETHHeaderCellDataView {
    pub headers: Bytes,
}

impl ETHHeaderCellDataView {
    pub fn from_slice(slice: &[u8]) -> ETHHeaderCellDataView {
        ETHHeaderCellDataReader::verify(slice, false).expect("ETHHeaderCellDataReader verify slice fail");
        let data_reader = ETHHeaderCellDataReader::new_unchecked(slice);
        let headers = data_reader.headers().to_entity().as_bytes();
        ETHHeaderCellDataView{headers}
    }
}
