FORCE_CLI := ./offchain-modules/target/debug/force-eth-cli

.EXPORT_ALL_VARIABLES:

FORCE_CONFIG_PATH=${HOME}/.force-bridge/config.toml
RUST_BACKTRACE=1
RUST_LOG=warn,force=info

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
	${FORCE_CLI} init-config --project-path ${shell pwd}

deploy-ckb:
	${FORCE_CLI} deploy-ckb --type-id -k 0

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

init-multi-address:
	${FORCE_CLI} init-multi-sign-address -k 1 --multi-address  ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37

ckb2eth-relay:
	pm2 start --name ckb2eth-relay "${FORCE_CLI} ckb-relay -k 1 --per-amount 5"

eth2ckb-relay:
	pm2 start --name eth2ckb-relay "${FORCE_CLI} eth-relay -k 1 --multisig-privkeys 0 1"

start-relay: ckb2eth-relay eth2ckb-relay

restart-ckb2eth-relay:
	pm2 restart ckb2eth-relay

restart-eth2ckb-relay:
	pm2 restart eth2ckb-relay

restart-relay: restart-ckb2eth-relay restart-eth2ckb-relay

start-force-server:
	cd offchain-modules \
	&& pm2 start --name force-server "./target/debug/force-eth-cli server  --ckb-private-key-path 2 --eth-private-key-path 2 --listen-url 0.0.0.0:3003"

restart-force-server:
	pm2 restart force-server

start-services: start-relay start-force-server

restart-services: restart-relay restart-force-server

deploy-from-scratch: deploy-contracts init-light-client start-services

start-demo-services:
	bash offchain-modules/start-services.sh

stop-demo-services:
	bash offchain-modules/stop-services.sh

setup-dev-env: build-all start-docker-network deploy-ckb-sudt init-multi-address deploy-eth deploy-erc20 start-demo-services

close-dev-env: stop-demo-services remove-docker-network

integration-ci: setup-dev-env demo-crosschain

local-ci:
	make close-dev-env
	test -f ~/.force-bridge/config.toml && mv ~/.force-bridge/config.toml ~/.force-bridge/config_bak_`date "+%Y%m%d-%H%M%S"`.toml || echo 'config not exist'
	cd offchain-modules && cargo build
	make init-config
	make integration-ci

github-ci:
	cd offchain-modules && cargo build
	make init-config
	make integration-ci

demo-crosschain:
	bash demo/crosschain.sh

testnet-demo: deploy-contracts deploy-erc20 start-demo-services demo-crosschain

build-docker:
	make -C docker build

fmt:
	make -C offchain-modules fmt
	make -C ckb-contracts fmt

.PHONY: demo
