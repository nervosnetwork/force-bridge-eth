#![cfg_attr(not(feature = "std"), no_std)]

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
    } else {
        extern crate alloc;
    }
}

pub mod traits;
#[cfg(not(feature = "std"))]
pub mod chain;
#[cfg(feature = "std")]
pub mod mock;

