use crate::ckb_constants::*;
// re-export to maintain compatible with old versions
pub use crate::error::SysError;

#[cfg(target_arch = "riscv64")]
#[link(name = "ckb-syscall")]
extern "C" {
    fn syscall(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64, a7: u64) -> u64;
}

#[cfg(not(target_arch = "riscv64"))]
fn syscall(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64, a7: u64) -> u64 {
    return u64::MAX;
}

/// Exit, this script will be terminated after the exit syscall.
/// exit code `0` represents verification is success, others represent error code.
pub fn exit(code: i8) -> ! {
    unsafe { syscall(code as u64, 0, 0, 0, 0, 0, 0, SYS_EXIT) };
    loop {}
}

/// Load data
/// Return data length or syscall error
fn syscall_load(
    buf_ptr: *mut u8,
    len: usize,
    offset: usize,
    a3: u64,
    a4: u64,
    a5: u64,
    a6: u64,
    syscall_num: u64,
) -> Result<usize, SysError> {
    let mut actual_data_len = len;
    let len_ptr: *mut usize = &mut actual_data_len;
    let ret = unsafe {
        syscall(
            buf_ptr as u64,
            len_ptr as u64,
            offset as u64,
            a3,
            a4,
            a5,
            a6,
            syscall_num,
        )
    };
    SysError::build_syscall_result(ret, len, actual_data_len)
}

/// Load transaction hash
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
///
/// # Example
///
/// ```
/// let mut tx_hash = [0u8; 32];
/// let len = load_tx_hash(&mut tx_hash, 0).unwrap();
/// assert_eq!(len, tx_hash.len());
/// ```
pub fn load_tx_hash(buf: &mut [u8], offset: usize) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        0,
        0,
        0,
        0,
        SYS_LOAD_TX_HASH,
    )
}

/// Load script hash
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
///
/// # Example
///
/// ```
/// let mut script_hash = [0u8; 32];
/// let len = load_script_hash(&mut script_hash, 0).unwrap();
/// assert_eq!(len, script_hash.len());
/// ```
pub fn load_script_hash(buf: &mut [u8], offset: usize) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        0,
        0,
        0,
        0,
        SYS_LOAD_SCRIPT_HASH,
    )
}

/// Load cell
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index of cell
/// * `source` - source of cell
pub fn load_cell(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        0,
        0,
        SYS_LOAD_CELL,
    )
}

/// Load input
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index of cell
/// * `source` - source of cell
pub fn load_input(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        0,
        0,
        SYS_LOAD_INPUT,
    )
}

/// Load header
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index of cell or header
/// * `source` - source
pub fn load_header(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        0,
        0,
        SYS_LOAD_HEADER,
    )
}

/// Load witness
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index of cell
/// * `source` - source
pub fn load_witness(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        0,
        0,
        SYS_LOAD_WITNESS,
    )
}

/// Load transaction
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
pub fn load_transaction(buf: &mut [u8], offset: usize) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        0,
        0,
        0,
        0,
        SYS_LOAD_TRANSACTION,
    )
}

/// Load cell by field
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index of cell
/// * `source` - source of cell
/// * `field` - field of cell
///
/// # Example
///
/// ```
/// let mut buf = [0u8; size_of::<u64>()];
/// let len = load_cell_by_field(&mut buf, 0, 0, Source::GroupInput, CellField::Capacity).unwrap();
/// assert_eq!(len, buf.len());
/// ```
pub fn load_cell_by_field(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
    field: CellField,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        field as u64,
        0,
        SYS_LOAD_CELL_BY_FIELD,
    )
}

/// Load header by field
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index
/// * `source` - source
/// * `field` - field
///
/// # Example
///
/// ```
/// let mut buf = [0u8; 8];
/// let len = load_header_by_field(&mut buf, 0, index, source, HeaderField::EpochNumber)?;
/// debug_assert_eq!(len, buf.len());
/// ```
pub fn load_header_by_field(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
    field: HeaderField,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        field as u64,
        0,
        SYS_LOAD_HEADER_BY_FIELD,
    )
}

/// Load input by field
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index
/// * `source` - source
/// * `field` - field
///
/// # Example
///
/// ```
/// let mut buf = [0u8; 8];
/// let len = load_input_by_field(&mut buf, 0, index, source, InputField::Since)?;
/// debug_assert_eq!(len, buf.len());
/// ```
pub fn load_input_by_field(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
    field: InputField,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        field as u64,
        0,
        SYS_LOAD_INPUT_BY_FIELD,
    )
}

/// Load cell data, read cell data
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
/// * `index` - index
/// * `source` - source
pub fn load_cell_data(
    buf: &mut [u8],
    offset: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        index as u64,
        source as u64,
        0,
        0,
        SYS_LOAD_CELL_DATA,
    )
}

/// Load script
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf` - a writable buf used to receive the data
/// * `offset` - offset
pub fn load_script(buf: &mut [u8], offset: usize) -> Result<usize, SysError> {
    syscall_load(
        buf.as_mut_ptr(),
        buf.len(),
        offset,
        0,
        0,
        0,
        0,
        SYS_LOAD_SCRIPT,
    )
}

/// Output debug message
///
/// You should use the macro version syscall: `debug!`
///
/// # Arguments
///
/// * `s` - string to output
pub fn debug(mut s: alloc::string::String) {
    s.push('\0');
    let c_str = s.into_bytes();
    unsafe {
        syscall(c_str.as_ptr() as u64, 0, 0, 0, 0, 0, 0, SYS_DEBUG);
    }
}

/// Load cell data, read cell data
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf_ptr` - a writable pointer used to receive the data
/// * `len` - length that the `buf_ptr` can receives.
/// * `offset` - offset
/// * `index` - index
/// * `source` - source
pub fn load_cell_data_raw(
    buf_ptr: *mut u8,
    len: usize,
    offset: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    syscall_load(
        buf_ptr,
        len,
        offset,
        index as u64,
        source as u64,
        0,
        0,
        SYS_LOAD_CELL_DATA,
    )
}

/// Load cell code, read cell data as executable code
///
/// Return the loaded data length or a syscall error
///
/// # Arguments
///
/// * `buf_ptr` - a writable pointer used to receive the data
/// * `len` - length that the `buf_ptr` can receives.
/// * `content_offset` - offset
/// * `content_size` - read length
/// * `index` - index
/// * `source` - source
pub fn load_cell_code(
    buf_ptr: *mut u8,
    len: usize,
    content_offset: usize,
    content_size: usize,
    index: usize,
    source: Source,
) -> Result<usize, SysError> {
    let ret = unsafe {
        syscall(
            buf_ptr as u64,
            len as u64,
            content_offset as u64,
            content_size as u64,
            index as u64,
            source as u64,
            0,
            SYS_LOAD_CELL_DATA_AS_CODE,
        )
    };
    SysError::build_syscall_result(ret, len, len)
}
