use std::env;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    // ckb-std only supports riscv64 target arch
    // but we can still use cargo check under other archs
    if target_arch == "riscv64" {
        cc::Build::new()
            .file("src/asm/syscall.S")
            .compile("ckb-syscall");
    }
}
