ci: modules-ci demo

modules-ci: ckb-contracts-ci eth-contracts-ci offchain-modules-ci

offchain-modules-ci:
	cd offchain-modules && make ci

ckb-contracts-ci:
	make -C ckb-contracts ci

eth-contracts-ci:
	cd eth-contracts && yarn test

demo:
	cd ckb-contracts && capsule build --release
	cd offchain-modules && cargo build
	bash demo/demo.sh

.PHONY: demo