#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

mod code_hashes;

use alloc::vec;
use alloc::vec::Vec;
use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::{
    ckb_constants::*, ckb_types::prelude::*, debug, default_alloc, entry, error::SysError,
    high_level, syscalls, dynamic_loading
};
use core::mem::{size_of, size_of_val};
use code_hashes::CODE_HASH_SHARED_LIB;

fn new_blake2b() -> Blake2b {
    const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";
    Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

fn test_basic() {
    let v = vec![0u8; 42];
    debug!("{:?}", v.len());
}

fn test_load_data() {
    let data = high_level::load_cell_data(0, Source::Output).unwrap();
    assert_eq!(data.len(), 1000);
    assert!(data.iter().all(|&b| b == 42));
}

fn test_load_cell_field() {
    let mut buf = [0u8; size_of::<u64>()];
    let len = syscalls::load_cell_by_field(&mut buf, 0, 0, Source::GroupInput, CellField::Capacity)
        .unwrap();
    assert_eq!(len, buf.len());
    let capacity = u64::from_le_bytes(buf);
    let capacity2 = high_level::load_cell_capacity(0, Source::GroupInput).unwrap();
    assert_eq!(capacity, capacity2);
    debug!("input capacity {}", capacity);
}

fn test_load_tx_hash() {
    let mut tx_hash = [0u8; 32];
    let len = syscalls::load_tx_hash(&mut tx_hash, 0).unwrap();
    assert_eq!(len, tx_hash.len());
    let tx_hash2 = high_level::load_tx_hash().unwrap();
    assert_eq!(&tx_hash, &tx_hash2);
    debug!("tx hash {:?}", tx_hash);
}

fn test_partial_load_tx_hash() {
    let mut tx_hash = [0u8; 32];
    let len = syscalls::load_tx_hash(&mut tx_hash, 0).unwrap();
    assert_eq!(len, tx_hash.len());
    assert_ne!(tx_hash, [0u8; 32]);

    // partial load ..16
    let mut buf = [0u8; 16];
    let err = syscalls::load_tx_hash(&mut buf, 0).unwrap_err();
    assert_eq!(err, SysError::LengthNotEnough(32));
    assert_eq!(buf[..], tx_hash[..16]);
    // partial load 16..
    let len = syscalls::load_tx_hash(&mut buf, 16).unwrap();
    assert_eq!(len, buf.len());
    assert_eq!(buf[..], tx_hash[16..]);
}

fn test_high_level_apis() {
    use high_level::*;

    let tx = load_transaction().unwrap();
    let output = load_cell(0, Source::Output).unwrap();
    let output2 = tx.raw().outputs().get(0).unwrap();
    assert_eq!(output.as_slice(), output2.as_slice());

    let script = load_script().unwrap();
    let lock_script = load_cell_lock(0, Source::Input).unwrap();
    assert_eq!(script.as_slice(), lock_script.as_slice());

    let lock_hash = load_cell_lock_hash(0, Source::Input).unwrap();
    let lock_hash2 = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(lock_script.as_slice());
        hasher.finalize(&mut buf);
        buf
    };
    assert_eq!(lock_hash, lock_hash2);

    let tx_hash = load_tx_hash().unwrap();
    let tx_hash2 = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(tx.raw().as_slice());
        hasher.finalize(&mut buf);
        buf
    };
    assert_eq!(tx_hash, tx_hash2);

    let inputs_capacity = QueryIter::new(load_cell, Source::Input)
        .map(|cell| cell.capacity().unpack())
        .sum::<u64>();
    let outputs_capacity = QueryIter::new(load_cell, Source::Output)
        .map(|cell| cell.capacity().unpack())
        .sum::<u64>();
    assert_eq!(inputs_capacity, outputs_capacity);
}

fn test_query() {
    use high_level::*;

    let outputs: Vec<_> = QueryIter::new(load_cell, Source::Output).collect();
    assert_eq!(outputs.len(), 2);

    let inputs: Vec<_> = QueryIter::new(load_input, Source::Input).collect();
    assert_eq!(inputs.len(), 1);

    let cell_deps: Vec<_> = QueryIter::new(load_cell, Source::CellDep).collect();
    assert_eq!(cell_deps.len(), 2);

    let header_deps: Vec<_> = QueryIter::new(load_header, Source::HeaderDep).collect();
    assert_eq!(header_deps.len(), 0);

    let witnesses: Vec<_> = QueryIter::new(load_witness_args, Source::Input).collect();
    assert_eq!(witnesses.len(), 0);

    let type_hashes: Option<Vec<_>> = QueryIter::new(load_cell_type_hash, Source::Input).collect::<Option<_>>();
    assert!(type_hashes.is_none());

    let type_scripts: Option<Vec<_>> = QueryIter::new(load_cell_type, Source::Input).collect::<Option<_>>();
    assert!(type_scripts.is_none());
}

fn test_dynamic_loading() {
    let mut context = dynamic_loading::CKBDLContext::<[u8; 64 * 1024]>::new();
    let lib = context.load(&CODE_HASH_SHARED_LIB).expect("load shared lib");

    unsafe {
        type Plus42 = unsafe extern "C" fn(n: usize) -> usize;
        let plus_42: dynamic_loading::Symbol<Plus42> = lib.get(b"plus_42").expect("find plus_42");
        assert_eq!(plus_42(13), 13 + 42);

        type Foo = unsafe extern "C" fn() -> *const u8;
        let foo: dynamic_loading::Symbol<Foo> = lib.get(b"foo").expect("find foo");
        let ptr = foo();
        let mut buf = [0u8; 3];
        buf.as_mut_ptr().copy_from(ptr, buf.len());
        assert_eq!(&buf[..], b"foo");
    }

    // load multiple libraries
    let mut size = size_of_val(&context) - lib.consumed_size();
    let mut offset = lib.consumed_size();
    let mut libs = Vec::new();

    for _i in 0..3 {
        let lib = context.load_with_offset(&CODE_HASH_SHARED_LIB, offset, size).expect("load shared lib");
        size -= lib.consumed_size();
        offset += lib.consumed_size();
        libs.push(lib);
    }

    unsafe {
        type Plus42 = unsafe extern "C" fn(n: usize) -> usize;
        type Foo = unsafe extern "C" fn() -> *const u8;

        for lib in libs {
            let plus_42: dynamic_loading::Symbol<Plus42> = lib.get(b"plus_42").expect("find plus_42");
            assert_eq!(plus_42(13), 13 + 42);

            let foo: dynamic_loading::Symbol<Foo> = lib.get(b"foo").expect("find foo");
            let ptr = foo();
            let mut buf = [0u8; 3];
            buf.as_mut_ptr().copy_from(ptr, buf.len());
            assert_eq!(&buf[..], b"foo");
        }
    }
}

#[no_mangle]
pub fn main() -> i8 {
    test_basic();
    test_load_data();
    test_load_cell_field();
    test_load_tx_hash();
    test_partial_load_tx_hash();
    test_high_level_apis();
    test_query();
    test_dynamic_loading();
    0
}

entry!(main);
default_alloc!();
