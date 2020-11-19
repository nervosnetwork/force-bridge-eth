ci: modules-ci integration-ci

modules-ci: ckb-contracts-ci eth-contracts-ci offchain-modules-ci

offchain-modules-ci:
	cd offchain-modules && make ci

ckb-contracts-ci:
	make -C ckb-contracts ci

eth-contracts-ci:
	cd eth-contracts && yarn test

demo-build:
	cd ckb-contracts && capsule build --release
	cd offchain-modules && cargo build

integration-ci: demo-build
	cd docker && docker-compose up -d
	bash demo/demo.sh
	cd docker && docker-compose stop

demo:
	bash demo/demo.sh

.PHONY: demo