#![cfg_attr(not(feature = "std"), no_std)]

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
        use ckb_std::high_level::load_tx_hash;
    }
}

pub trait CkbChainInterface {
    fn load_tx_hash(&self) -> [u8; 32];
}

pub trait ContractInterface<T: CkbChainInterface> {
    fn verify(chain: T) -> i8;
}

#[derive(Debug, Default)]
pub struct MockCKBChain {
    pub tx_hash: [u8; 32],
}

impl CkbChainInterface for MockCKBChain {
    fn load_tx_hash(&self) -> [u8; 32] {
        self.tx_hash
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
