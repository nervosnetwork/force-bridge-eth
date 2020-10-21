use super::{Adapter, ComplexData};
use ckb_std::error::SysError;

#[derive(Default, Clone)]
pub struct MockAdapter {
    pub tx_hash: [u8; 32],
    pub complex_data: ComplexData,
}

impl Adapter for MockAdapter {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError> {
        Ok(self.tx_hash)
    }

    fn get_complex_data(&self) -> Result<ComplexData, SysError> {
        Ok(self.complex_data.clone())
    }
}
