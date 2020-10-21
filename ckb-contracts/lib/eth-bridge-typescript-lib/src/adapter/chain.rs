use super::{Adapter, ComplexData};
use ckb_std::error::SysError;
use ckb_std::high_level::load_tx_hash;

pub struct ChainAdapter {}

impl Adapter for ChainAdapter {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError> {
        load_tx_hash()
    }

    fn get_complex_data(&self) -> Result<ComplexData, SysError> {
        unimplemented!()
    }
}
