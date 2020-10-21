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
    let tx = data_loader.load_tx_hash();
    debug!("tx: {:?}", &tx);
    return 0;
}

#[cfg(test)]
mod tests {
    use super::_verify;
    use crate::adapter::mock::MockAdapter;

    #[test]
    fn it_works() {
        let mock_adapter = MockAdapter::default();
        let return_code = _verify(mock_adapter);
        assert_eq!(return_code, 0);
    }
}
