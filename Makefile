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

integration-ci: setup-dev-env demo-crosschain

demo-crosschain:
	bash demo/crosschain.sh

build-docker:
	make -C docker build

fmt:
	make -C offchain-modules fmt
	make -C ckb-contracts fmt

.PHONY: demo
