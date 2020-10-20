pub use blake2b_rs::{Blake2b, Blake2bBuilder};

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

const BUF_SIZE: usize = 8 * 1024;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

fn main() {
    let out_path = Path::new("src").join("code_hashes.rs");
    let mut out_file = BufWriter::new(File::create(&out_path).expect("create code_hashes.rs"));

    let name = "shared-lib";
    let path = format!("../shared-lib/{}.so", name);

    let mut buf = [0u8; BUF_SIZE];

    // build hash
    let mut blake2b = new_blake2b();
    let mut fd = File::open(&path).expect("open file");
    loop {
        let read_bytes = fd.read(&mut buf).expect("read file");
        if read_bytes > 0 {
            blake2b.update(&buf[..read_bytes]);
        } else {
            break;
        }
    }

    let mut hash = [0u8; 32];
    blake2b.finalize(&mut hash);

    write!(
        &mut out_file,
        "pub const {}: [u8; 32] = {:?};\n",
        format!("CODE_HASH_{}", name.to_uppercase().replace("-", "_")),
        hash
    )
        .expect("write to code_hashes.rs");
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

