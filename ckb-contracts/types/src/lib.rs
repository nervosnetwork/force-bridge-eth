#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;
pub mod convert;
pub mod eth_recipient_cell;
pub mod generated;
pub mod eth_header_cell;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}
