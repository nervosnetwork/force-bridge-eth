use ckb_std::error::SysError;
use crate::traits::CkbChainInterface;

#[derive(Debug, Default)]
pub struct MockCKBChain {
    pub tx_hash: [u8; 32],
}

impl CkbChainInterface for MockCKBChain {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError> {
        Ok(self.tx_hash)
    }
}
