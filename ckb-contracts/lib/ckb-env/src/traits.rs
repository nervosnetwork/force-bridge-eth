use ckb_std::error::SysError;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
    }
}

pub trait CkbChainInterface {
    fn load_tx_hash(&self) -> Result<[u8; 32], SysError>;
    // fn load_script_hash(&self) -> Result<[u8; 32], String>;
    // fn load_cell(index: usize, source: Source) -> Result<CellOutput, SysError>;
}

pub trait ContractInterface<T: CkbChainInterface> {
    fn verify(chain: T) -> i8;
}
