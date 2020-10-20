
all: test-contract-lib build

build:
	capsule build
	capsule test

test-contract-lib:
	cd lib/eth-bridge-typescript-lib && cargo test

.PHONY: build