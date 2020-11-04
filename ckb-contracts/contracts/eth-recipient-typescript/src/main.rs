//! Generated by capsule
//!
//! `main.rs` is used to define rust lang items and modules.
//! See `entry.rs` for the `main` function.
//! See `error.rs` for the `Error` type.

#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

use ckb_std::default_alloc;
use eth_bridge_typescript_lib::verify;

default_alloc!();

#[alloc_error_handler]
fn oom_handler(_layout: alloc::alloc::Layout) -> ! {
    panic!("Out of memory")
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let f: fn() -> i8 = verify;
    ckb_std::syscalls::exit(f())
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

/// Fix symbol missing
#[no_mangle]
pub extern "C" fn abort() {
    panic!("abort!");
}

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    {
        use alloc::format;

        let mut s = alloc::string::String::new();
        if let Some(p) = panic_info.payload().downcast_ref::<&str>() {
            s.push_str(&format!("panic occurred: {:?}", p));
        } else {
            s.push_str(&format!("panic occurred:"));
        }
        if let Some(m) = panic_info.message() {
            s.push_str(&format!(" {:?}", m));
        }
        if let Some(location) = panic_info.location() {
            s.push_str(&format!(
                ", in file {}:{}",
                location.file(),
                location.line()
            ));
        } else {
            s.push_str(&format!(", but can't get location information..."));
        }

        ckb_std::syscalls::debug(s);
    }
    ckb_std::syscalls::exit(-1)
}
