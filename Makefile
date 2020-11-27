ci: modules-ci integration-ci

modules-ci: ckb-contracts-ci eth-contracts-ci offchain-modules-ci

offchain-modules-ci:
	make -C offchain-modules ci

ckb-contracts-ci:
	make -C ckb-contracts ci

eth-contracts-ci:
	cd eth-contracts && yarn test

demo-build:
	mkdir -p demo/contracts demo/data demo/bin
	cd ckb-contracts && capsule build --release
	cp ckb-contracts/build/release/* demo/contracts/
	cp offchain-modules/cli/deps/simple_udt demo/contracts/
	cp -r offchain-modules/vendor demo/
	cp offchain-modules/data/dag_merkle_roots.json demo/data/dag_merkle_roots.json
	cd offchain-modules/eth-proof && npm install
	cp -r offchain-modules/eth-proof demo/
	cd offchain-modules && cargo build #--release
	cp offchain-modules/target/debug/force-eth-cli demo/bin/
	cd eth-contracts && yarn install

integration-ci: demo-build demo-start-deamon demo-run-crosschain

demo-clear:
	rm -rf demo/{bin,contracts,data,.force-bridge-cli-config.toml}

fmt:
	make -C offchain-modules fmt
	make -C ckb-contracts fmt

demo-init-deamon: demo-clear-deamon demo-start-deamon

demo-start-deamon:
	cd docker && docker-compose up -d
	bash demo/vendor/init_eth2ckb_relayer_key.sh
	bash demo/demo-start-deamon.sh

demo-run-crosschain:
	bash demo/crosschain.sh

demo-clear-deamon:
	bash demo/clean.sh
	cd docker && docker-compose down
	rm -rf ${HOME}/.ckb-cli/index-v1

.PHONY: demo
