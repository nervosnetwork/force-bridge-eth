
ci: test-contract
	cargo fmt -- --check
	cargo clippy -- -D warnings
	cd contracts/eth-bridge-lockscript && cargo fmt -- --check
	cd contracts/eth-light-client-typescript && cargo fmt -- --check
	cd contracts/eth-recipient-typescript && cargo fmt -- --check

fmt:
	cargo fmt
	cd contracts/eth-bridge-lockscript && cargo fmt
	cd contracts/eth-light-client-typescript && cargo fmt
	cd contracts/eth-recipient-typescript && cargo fmt

build-contract:
	capsule build

test-contract: build-contract
	cargo test -- --nocapture

test-contract-lib:
	cd lib/eth-bridge-typescript-lib && cargo test -- --nocapture

check-contract-lib: test-contract-lib
	cd lib/eth-bridge-typescript-lib && cargo fmt -- --check && cargo clippy -- -D warnings

.PHONY: ci
