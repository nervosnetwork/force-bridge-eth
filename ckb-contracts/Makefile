
ci: test-contract
	cargo fmt -- --check
	cargo clippy -- -D warnings

fmt:
	cd lib/eth-bridge-typescript-lib && cargo fmt
	cd contracts/eth-bridge-typescript && cargo fmt

build-contract:
	capsule build

test-contract: build-contract
	cargo test -- --nocapture

test-contract-lib:
	cd lib/eth-bridge-typescript-lib && cargo test -- --nocapture

check-contract-lib: test-contract-lib
	cd lib/eth-bridge-typescript-lib && cargo fmt -- --check && cargo clippy -- -D warnings

.PHONY: ci
