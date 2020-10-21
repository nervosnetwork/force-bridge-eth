
all: test-contract-lib build

fmt:
	cd lib/eth-bridge-typescript-lib && cargo fmt
	cd contracts/eth-bridge-typescript && cargo fmt

build:
	capsule build
	capsule test

test-contract-lib:
	cd lib/eth-bridge-typescript-lib && cargo test -- --nocapture

.PHONY: build