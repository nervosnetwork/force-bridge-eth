FORCE_CLI := ./offchain-modules/target/debug/force-eth-cli

.EXPORT_ALL_VARIABLES:

FORCE_CONFIG_PATH=${HOME}/.force-bridge/config.toml
RUST_BACKTRACE=1
RUST_LOG=info,force=debug

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
	sleep 5

remove-docker-network:
	cd docker && docker-compose down

init-config:
	${FORCE_CLI} init --project-path ${shell pwd}

deploy-ckb:
	${FORCE_CLI} init --project-path . -f
	${FORCE_CLI} deploy-ckb -k 0

deploy-ckb-sudt:
	${FORCE_CLI} deploy-ckb --sudt -k 0

deploy-eth:
	cd eth-contracts \
	&& npx hardhat run ./scripts/deploy.js

deploy-erc20:
	cd eth-contracts \
	&& npx hardhat run ./scripts/deploy-erc20.js > ~/.force-bridge/erc20-contracts.json

deploy-contracts: deploy-ckb deploy-eth

init-light-client:
	${FORCE_CLI} init-ckb-light-contract -k 0 -f 500 -c 40000 --wait

ckb2eth-relay:
	pm2 start --name ckb2eth-relay "${FORCE_CLI} ckb-relay -k 1 --per-amount 5"

eth2ckb-relay:
	pm2 start --name eth2ckb-relay "${FORCE_CLI} eth-relay -k 1"

start-relay: ckb2eth-relay eth2ckb-relay

restart-ckb2eth-relay:
	pm2 restart ckb2eth-relay

restart-eth2ckb-relay:
	pm2 restart eth2ckb-relay

restart-relay: restart-ckb2eth-relay restart-eth2ckb-relay

start-force-server:
	pm2 start --name force-server "${FORCE_CLI} server  --ckb-private-key-path 2 --eth-private-key-path 2 --listen-url 0.0.0.0:3003"

restart-force-server:
	pm2 restart force-server

start-services: start-relay start-force-services

restart-services: restart-relay restart-force-server

deploy-from-scratch: deploy-contracts init-light-client start-services

start-demo-services:
	bash offchain-modules/start-services.sh

stop-demo-services:
	bash offchain-modules/stop-services.sh

setup-dev-env: build-all start-docker-network deploy-ckb-sudt deploy-eth deploy-erc20 start-demo-services

close-dev-env: stop-demo-services remove-docker-network

integration-ci: setup-dev-env demo-crosschain

demo-crosschain:
	bash demo/crosschain.sh

testnet-demo: deploy-contracts deploy-erc20 start-demo-services demo-crosschain

build-docker:
	make -C docker build

fmt:
	make -C offchain-modules fmt
	make -C ckb-contracts fmt

.PHONY: demo
