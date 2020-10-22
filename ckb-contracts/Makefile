
all: test-contract-lib test-contract

fmt:
	cd lib/eth-bridge-typescript-lib && cargo fmt
	cd contracts/eth-bridge-typescript && cargo fmt

test-contract:
	capsule build
	capsule test

test-contract-lib:
	cd lib/eth-bridge-typescript-lib && cargo test -- --nocapture

.PHONY: build