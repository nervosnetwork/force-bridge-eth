test: clean build patch
	RUST_LOG=debug cargo test -- --nocapture

build:
	make -C shared-lib shared-lib.so
	cd contract && cargo build

C := contract/target/riscv64imac-unknown-none-elf/debug/contract
patch:
	ckb-binary-patcher -i $C -o $C

clean:
	cd contract && cargo clean
