#![cfg_attr(not(feature = "std"), no_std)]

pub mod adapter;
pub mod debug;

pub use adapter::Adapter;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}

#[cfg(target_arch = "riscv64")]
pub fn verify() -> i8 {
    let chain = adapter::chain::ChainAdapter {};
    _verify(chain)
}

pub fn _verify<T: Adapter>(data_loader: T) -> i8 {
    let tx = data_loader.load_tx_hash().expect("load tx hash failed");
    debug!("tx: {:?}", &tx);
    0
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::*;
    use ckb_std::error::SysError;

    #[test]
    fn mock_return_ok() {
        let mut mock = MockAdapter::new();
        mock.expect_load_tx_hash()
            .times(1)
            .returning(|| Ok([0u8; 32]));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }

    #[test]
    #[should_panic]
    fn mock_return_err() {
        let mut mock = MockAdapter::new();
        mock.expect_load_tx_hash()
            .times(1)
            .returning(|| Err(SysError::Encoding));
        let return_code = _verify(mock);
        assert_eq!(return_code, 0);
    }
}
