ci: modules-ci integration-ci

modules-ci: ckb-contracts-ci eth-contracts-ci offchain-modules-ci

offchain-modules-ci:
	make -C offchain-modules ci

ckb-contracts-ci:
	make -C ckb-contracts ci

eth-contracts-ci:
	cd eth-contracts && yarn test

build-all:
	cd ckb-contracts && capsule build --release
	cd eth-contracts && yarn install
	cd offchain-modules && cargo build
	cd offchain-modules/eth-proof && npm install

start-docker-network:
	cd docker && docker-compose up -d

remove-docker-network:
	cd docker && docker-compose down

deploy-contracts:
	bash offchain-modules/deploy.sh

start-offchain-services:
	bash offchain-modules/start-services.sh

stop-offchain-services:
	bash offchain-modules/stop-services.sh

setup-dev-env: build-all start-docker-network deploy-contracts start-offchain-services

close-dev-env: stop-offchain-services remove-docker-network

demo-build: build-all
	mkdir -p demo/{contracts,data,bin} && cp ckb-contracts/build/release/* demo/contracts
	cp offchain-modules/cli/deps/simple_udt demo/contracts
	cp -r offchain-modules/vendor demo
	cp offchain-modules/data/dag_merkle_roots.json demo/data/dag_merkle_roots.json
	cp -r offchain-modules/eth-proof demo
	cp offchain-modules/target/debug/force-eth-cli demo/bin

integration-ci: demo-build
	cd docker && docker-compose up -d
	bash demo/demo.sh
	#cd docker && docker-compose stop

demo-clear:
	rm -rf demo/{bin,contracts,data,.force-bridge-cli-config.toml}

demo:
	bash demo/demo.sh

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

.PHONY: demo
