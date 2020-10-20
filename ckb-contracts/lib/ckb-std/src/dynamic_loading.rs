//! This module supports dynamic loading a library from an on-chain cell.
//!
//! # Pre-requirement
//!
//! * Shared library: a standard ELF shared library, usually with a `.so` file extension ([example of how to create a shared library](https://github.com/nervosnetwork/ckb-miscellaneous-scripts)).
//! * Shared library cell: deploy the shared library to the chain.
//! * Transaction: use the CellDep field reference to the shared library cell.
//!
//! # Example
//!
//! Shared library(C)
//!
//! ```
//! typedef unsigned long size_t;
//!
//! __attribute__((visibility("default"))) int
//! plus_42(size_t num) {
//!   return 42 + num;
//! }
//!
//! __attribute__((visibility("default"))) char *
//! foo() {
//!   return "foo";
//! }
//! ```
//!
//! Rust contract
//!
//! ```
//! use ckb_std::dynamic_loading::{CKBDLContext, Symbol};
//!
//! /// code hash of the shared library
//! pub const CODE_HASH_SHARED_LIB: [u8; 32] = [235, 179, 185, 44, 159, 213, 242, 94, 42, 196, 68, 5, 213, 248, 71, 106, 136, 183, 99, 125, 37, 214, 63, 59, 57, 87, 65, 80, 177, 92, 23, 255];
//!
//! // create a dynamic loading context instance
//! // we use [u8; 64 * 1024] as the buffer to receive the code, the size of the buffer must be
//! // aligned to PAGE_SIZE 4096, otherwise will return an error.
//! //
//! // NOTICE: CKB-VM using a W^X memory model, after loading code into memory pages, these pages can't
//! // be deallocated, which means we should never drop a CKBDLContext instance, otherwise a
//! // InvalidPermission error will occuer to terminate our script.
//! //
//! // [W^X memory model](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0003-ckb-vm/0003-ckb-vm.md#wx-memory)
//! let mut context = CKBDLContext::<[u8; 64 * 1024]>::new();
//!
//! // load a shared library from dep cells
//! let lib = context.load(&CODE_HASH_SHARED_LIB).expect("load shared lib");
//!
//! unsafe {
//!     type Plus42 = unsafe extern "C" fn(n: usize) -> usize;
//!     let plus_42: Symbol<Plus42> = lib.get(b"plus_42").expect("find plus_42");
//!     assert_eq!(plus_42(13), 13 + 42);
//!
//!     type Foo = unsafe extern "C" fn() -> *const u8;
//!     let foo: Symbol<Foo> = lib.get(b"foo").expect("find foo");
//!     let ptr = foo();
//!     let mut buf = [0u8; 3];
//!     buf.as_mut_ptr().copy_from(ptr, buf.len());
//!     assert_eq!(&buf[..], b"foo");
//! }
//! ```
//!
//! The core part of this module is inspired from
//! https://github.com/nervosnetwork/ckb-c-stdlib/blob/eae8c4c974ce68ca8062521747a16e8e59de755f/ckb_dlfcn.h
//!
//! The ELF parsing code is inspired from
//! https://github.com/riscv/riscv-pk/blob/master/pk/elf.h
//! original code is in BSD license.

use crate::ckb_constants::Source;
use crate::error::SysError;
use crate::high_level::find_cell_by_data_hash;
use crate::syscalls::{load_cell_code, load_cell_data_raw};
use core::cmp::{max, min};
use core::marker::PhantomData;
use core::mem::{size_of, MaybeUninit};

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

const SHT_STRTAB: usize = 3;
const SHT_RELA: usize = 4;
const SHT_DYNSYM: usize = 11;

#[repr(C)]
struct Elf64Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

const PT_LOAD: usize = 1;
const PF_X: usize = 1;

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[repr(C)]
struct Elf64Sym {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    st_size: u64,
}

const R_RISCV_RELATIVE: usize = 3;

#[repr(C)]
struct Elf64Rela {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

const RISCV_PGSIZE_SHIFT: usize = 12;
const RISCV_PGSIZE: usize = 1 << RISCV_PGSIZE_SHIFT; // 4096

/// roundup, use shift operator to reduce cycles
fn roundup_shift(a: usize, shift_n: usize) -> usize {
    (((a - 1) >> shift_n) + 1) << shift_n
}

/// Dynamic loading errors
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Create context error
    ContextFailure,
    /// Parse ELF header error
    InvalidElf,
    /// Memory not enough
    MemoryNotEnough,
    /// Can't find the cell
    CellNotFound,
    /// Invalid alignment
    InvalidAlign,
    /// Syscall error
    Sys(SysError),
}

impl From<SysError> for Error {
    fn from(error: SysError) -> Error {
        Error::Sys(error)
    }
}

/// Wrapper of dynamic loaded symbols
pub struct Symbol<T> {
    ptr: *const u8,
    phantom: PhantomData<T>,
}

impl<T> Symbol<T> {
    fn new(ptr: *const u8) -> Self {
        Symbol {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl<T> core::ops::Deref for Symbol<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(&self.ptr) }
    }
}

/// Dynamic loaded library
pub struct Library {
    dynsyms: *const Elf64Sym,
    dynstr: *const u8,
    dynsym_size: usize,
    base_addr: *const u8,
    consumed_size: usize,
}

impl Library {
    fn new() -> Self {
        Library {
            dynsyms: core::ptr::null(),
            dynstr: core::ptr::null(),
            dynsym_size: 0,
            base_addr: core::ptr::null(),
            consumed_size: 0,
        }
    }

    /// Library consumed size
    pub fn consumed_size(&self) -> usize {
        self.consumed_size
    }

    /// # Unsafe
    ///
    /// Undefined behavior will happen if the type S not match the type of symbol in the shared
    /// library
    ///
    /// Return None if not found the symbol
    pub unsafe fn get<S>(&self, symbol: &[u8]) -> Option<Symbol<S>> {
        unsafe fn cmp_raw_str(ptr: *const u8, s: &[u8]) -> bool {
            let mut i = 0;
            for c in s {
                let sym_c = *ptr.add(i);
                // return false if symbol string is end
                if sym_c == 0 {
                    return false;
                }
                if &sym_c != c {
                    return false;
                }
                i += 1;
            }
            let term_c = *ptr.add(i);
            // return false if symbol string is not terminated
            term_c == 0
        }

        for i in 0..self.dynsym_size {
            let sym = self.dynsyms.add(i);
            let str_ptr = self.dynstr.add((*sym).st_name as usize);
            if cmp_raw_str(str_ptr, symbol) {
                let sym_ptr = self.base_addr.add((*sym).st_value as usize);
                return Some(Symbol::new(sym_ptr));
            }
        }

        return None;
    }
}

/// Dynamic loading context
/// T represent a buffer type, for example: [u8; 64 * 1024], the size of T must aligned with
/// PAGE_SIZE 4096.
#[repr(C)]
#[repr(align(4096))]
pub struct CKBDLContext<T>(T);

impl<T> CKBDLContext<T> {
    /// Create instance of dynamic loading context
    pub fn new() -> Self {
        unsafe { MaybeUninit::<CKBDLContext<T>>::uninit().assume_init() }
    }

    /// Load a shared library from dep cells
    /// See module level documentation for details
    pub fn load<'a>(&'a mut self, dep_cell_data_hash: &[u8]) -> Result<Library, Error> {
        self.load_with_offset(dep_cell_data_hash, 0, size_of::<CKBDLContext<T>>())
    }

    /// Load a shared library from dep cells to specified buffer offset.
    /// See module level documentation for details
    ///
    /// This function is used for loading multiple libraries.
    ///
    /// # Example
    ///
    /// ```
    /// let mut context = CKBDLContext::<[u8; 64 * 1024]>::new();
    /// let mut size = size_of_val(&context);
    /// let mut offset = 0;
    ///
    /// let lib1 = context.load(&CODE_HASH_LIB_1).expect("load shared lib");
    /// size -= lib1.consumed_size()
    /// offset += lib1.consumed_size()
    ///
    /// let lib2 = context.load_with_offset(&CODE_HASH_LIB_2, offset, size).expect("load shared lib");
    /// size -= lib2.consumed_size()
    /// offset += lib2.consumed_size()
    ///
    /// let lib3 = context.load_with_offset(&CODE_HASH_LIB_3, offset, size).expect("load shared lib");
    /// ```
    pub fn load_with_offset<'a>(
        &'a mut self,
        dep_cell_data_hash: &[u8],
        offset: usize,
        size: usize,
    ) -> Result<Library, Error> {
        if size_of::<Library>() > RISCV_PGSIZE || size < RISCV_PGSIZE {
            return Err(Error::ContextFailure);
        }

        // size must aligned to page size
        if ((size >> RISCV_PGSIZE_SHIFT) << RISCV_PGSIZE_SHIFT) != size {
            return Err(Error::InvalidAlign);
        }

        unsafe {
            // initialize context
            let aligned_size = size;
            let aligned_addr = (&mut self.0 as *mut T).cast::<u8>().add(offset);
            let mut library = Library::new();
            library.base_addr = aligned_addr;

            let index = find_cell_by_data_hash(dep_cell_data_hash, Source::CellDep)?
                .ok_or(Error::CellNotFound)?;

            // Basic ELF header parsing
            let mut hdr = MaybeUninit::<Elf64Ehdr>::uninit().assume_init();
            let len = size_of::<Elf64Ehdr>();
            let loaded_len = {
                let elf_hdr_ptr = &mut hdr as *mut Elf64Ehdr;
                match load_cell_data_raw(elf_hdr_ptr.cast(), len, 0, index, Source::CellDep) {
                    Ok(len) => len,
                    Err(SysError::LengthNotEnough(_)) => len,
                    Err(err) => return Err(err.into()),
                }
            };
            if loaded_len < len {
                return Err(Error::InvalidElf);
            }
            if (hdr.e_phentsize as usize != size_of::<Elf64Phdr>())
                || (hdr.e_shentsize as usize != size_of::<Elf64Shdr>())
                || (hdr.e_phnum > 16)
                || (hdr.e_shnum > 32)
            {
                return Err(Error::InvalidElf);
            }

            // Parse program headers and load relevant parts
            let mut program_hdrs = MaybeUninit::<[Elf64Phdr; 16]>::uninit().assume_init();
            let len = size_of::<Elf64Phdr>() * hdr.e_phnum as usize;
            let loaded_len = {
                let ptr = program_hdrs.as_mut_ptr();
                match load_cell_data_raw(
                    ptr.cast(),
                    len,
                    hdr.e_phoff as usize,
                    index,
                    Source::CellDep,
                ) {
                    Ok(len) => len,
                    Err(SysError::LengthNotEnough(_)) => len,
                    Err(err) => return Err(err.into()),
                }
            };
            if loaded_len < len {
                return Err(Error::InvalidElf);
            }
            let mut max_consumed_size = 0;
            for ph in &program_hdrs[0..hdr.e_phnum as usize] {
                if ph.p_type as usize == PT_LOAD && ph.p_memsz > 0 {
                    if (ph.p_flags as usize & PF_X) != 0 {
                        let prepad = ph.p_vaddr as usize % RISCV_PGSIZE;
                        let vaddr = ph.p_vaddr as usize - prepad;
                        let memsz = roundup_shift(prepad + ph.p_memsz as usize, RISCV_PGSIZE_SHIFT);
                        let size = vaddr + memsz;
                        if size > aligned_size {
                            return Err(Error::MemoryNotEnough);
                        }
                        load_cell_code(
                            aligned_addr.add(vaddr),
                            memsz,
                            ph.p_offset as usize,
                            ph.p_filesz as usize,
                            index,
                            Source::CellDep,
                        )?;
                        max_consumed_size = max(max_consumed_size, vaddr + memsz);
                    } else {
                        let filesz = ph.p_filesz as usize;
                        let size = ph.p_vaddr as usize + filesz;
                        let consumed_end: usize = roundup_shift(size, RISCV_PGSIZE_SHIFT);
                        if consumed_end > aligned_size {
                            return Err(Error::MemoryNotEnough);
                        }
                        let loaded_len = match load_cell_data_raw(
                            aligned_addr.add(ph.p_vaddr as usize),
                            filesz,
                            ph.p_offset as usize,
                            index,
                            Source::CellDep,
                        ) {
                            Ok(len) => len,
                            Err(SysError::LengthNotEnough(_)) => filesz,
                            Err(err) => return Err(err.into()),
                        };
                        if loaded_len < filesz {
                            return Err(Error::InvalidElf);
                        }
                        max_consumed_size = max(max_consumed_size, consumed_end);
                    }
                }
            }

            // Parse sectioin header & relocation headers,
            // Perform necessary relocations.

            let mut section_hdrs = MaybeUninit::<[Elf64Shdr; 32]>::uninit().assume_init();
            let len = size_of::<Elf64Shdr>() * hdr.e_shnum as usize;
            let loaded_len = {
                let ptr = section_hdrs.as_mut_ptr();
                match load_cell_data_raw(
                    ptr.cast(),
                    len,
                    hdr.e_shoff as usize,
                    index,
                    Source::CellDep,
                ) {
                    Ok(len) => len,
                    Err(SysError::LengthNotEnough(_)) => len,
                    Err(err) => return Err(err.into()),
                }
            };
            if loaded_len < len {
                return Err(Error::InvalidElf);
            }

            // First load shstrtab tab, this is temporary code only needed in ELF loading
            // phase here.
            let shshrtab = &section_hdrs[hdr.e_shstrndx as usize];
            let mut shrtab = MaybeUninit::<[u8; 4096]>::uninit().assume_init();
            if shshrtab.sh_size > 4096 {
                return Err(Error::InvalidElf);
            }
            let shrtab_len = shshrtab.sh_size as usize;
            let _loaded_len = {
                let ptr = shrtab.as_mut_ptr();
                match load_cell_data_raw(
                    ptr.cast(),
                    shrtab_len,
                    shshrtab.sh_offset as usize,
                    index,
                    Source::CellDep,
                ) {
                    Ok(len) => len,
                    Err(SysError::LengthNotEnough(_)) => len,
                    Err(err) => return Err(err.into()),
                }
            };
            if shrtab_len < shshrtab.sh_size as usize {
                return Err(Error::InvalidElf);
            }
            for sh in &section_hdrs[0..hdr.e_shnum as usize] {
                if sh.sh_type as usize == SHT_RELA {
                    if sh.sh_entsize as usize != size_of::<Elf64Rela>() {
                        return Err(Error::InvalidElf);
                    }
                    let mut relocation_size = (sh.sh_size / sh.sh_entsize) as usize;
                    let mut current_offset = sh.sh_offset as usize;
                    while relocation_size > 0 {
                        let mut relocations =
                            MaybeUninit::<[Elf64Rela; 64]>::uninit().assume_init();
                        let load_size = min(relocation_size, 64) as usize;
                        let load_length = load_size * size_of::<Elf64Rela>();
                        let loaded_len = {
                            let ptr = relocations.as_mut_ptr();
                            match load_cell_data_raw(
                                ptr.cast(),
                                load_length,
                                current_offset,
                                index,
                                Source::CellDep,
                            ) {
                                Ok(len) => len,
                                Err(SysError::LengthNotEnough(_)) => load_length,
                                Err(err) => return Err(err.into()),
                            }
                        };
                        if loaded_len < load_length {
                            return Err(Error::InvalidElf);
                        }
                        relocation_size -= load_size;
                        current_offset += len;
                        for r in &relocations[0..load_size] {
                            if r.r_info as usize != R_RISCV_RELATIVE {
                                // Only relative relocation is supported now, we might add more
                                // later
                                return Err(Error::InvalidElf);
                            }
                            aligned_addr
                                .add(r.r_offset as usize)
                                .cast::<u64>()
                                .write_unaligned(
                                    aligned_addr.offset(r.r_addend as isize) as usize as u64
                                );
                        }
                    }
                } else if sh.sh_type as usize == SHT_DYNSYM {
                    // We assume one ELF file only has one DYNSYM section now
                    if sh.sh_entsize as usize != size_of::<Elf64Sym>() {
                        return Err(Error::InvalidElf);
                    }
                    library.dynsyms = aligned_addr.add(sh.sh_offset as usize).cast();
                    library.dynsym_size = (sh.sh_size / sh.sh_entsize) as usize;
                } else if sh.sh_type as usize == SHT_STRTAB {
                    let s = b".dynstr";
                    if &shrtab[sh.sh_name as usize..sh.sh_name as usize + s.len()] == s {
                        library.dynstr = aligned_addr.add(sh.sh_offset as usize);
                    }
                }
            }

            if library.dynsyms.is_null() || library.dynstr.is_null() {
                return Err(Error::InvalidElf);
            }
            let consumed_size = max_consumed_size + RISCV_PGSIZE;
            library.consumed_size = consumed_size;
            return Ok(library);
        }
    }
}
