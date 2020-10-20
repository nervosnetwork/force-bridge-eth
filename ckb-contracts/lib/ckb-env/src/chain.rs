use crate::traits::CkbChainInterface;
use ckb_std::error::SysError;
use ckb_std::high_level::load_tx_hash;

pub struct CKBChain {}

impl CkbChainInterface for CKBChain {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError> {
        load_tx_hash()
    }
}
