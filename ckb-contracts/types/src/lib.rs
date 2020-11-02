#![cfg_attr(not(feature = "std"), no_std)]

pub mod convert;
pub mod eth_recipient_cell;
pub mod generated;

pub use generated::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}
