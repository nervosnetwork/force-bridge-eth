
all: check-contract-lib test-capsule

fmt:
	cd lib/eth-bridge-typescript-lib && cargo fmt
	cd contracts/eth-bridge-typescript && cargo fmt

test-capsule:
	capsule build
	capsule test

test-contract-lib:
	cd lib/eth-bridge-typescript-lib && cargo test -- --nocapture

check-contract-lib: test-contract-lib
	cd lib/eth-bridge-typescript-lib && cargo fmt -- --check && cargo clippy -- -D warnings

.PHONY: build