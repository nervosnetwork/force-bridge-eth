#![warn(missing_docs)]
#![no_std]

//! This crate is part of the `eth-spv` project.
//!
//! It contains a collection of Rust functions and structs for working with
//! eth data structures. Basically, these tools help you parse, inspect,
//! and authenticate eth transactions.

/// `eth_types` exposes simple types for on-chain evaluation of SPV proofs.
pub mod eth_types;

/// `ethspv` provides higher-levels of abstraction for evaluating SPV proofs.
pub mod ethspv;

