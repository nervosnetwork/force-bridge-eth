use ethereum_types::U256;
use std::convert::TryInto;
// cfg_if::cfg_if! {
//     if #[cfg(feature = "std")] {
//     } else {
//         use core::convert::TryInto;
//     }
// }

/// The precision of eth token is usually 10^18, and ckb is 10^8
pub const ETH_CKB_RATE: u128 = 10_000_000_000;

pub fn eth_to_ckb_amount(n: U256) -> Result<u128, &'static str> {
    (n / ETH_CKB_RATE).try_into()
}

pub fn ckb_to_eth_amount(n: u128) -> U256 {
    U256::from(n) * U256::from(ETH_CKB_RATE)
}
