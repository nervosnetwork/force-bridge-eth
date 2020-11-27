#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
export FORCE_CONFIG_PATH="${DIR}"/demo/.force-bridge-cli-config.toml
DATA_DIR="${DIR}"/demo/data
mkdir -p "${DATA_DIR}"
FORCE_CLI="${DIR}"/demo/bin/force-eth-cli
FORTH_ETH_CONFIG_PATH="${DATA_DIR}"/force-eth-config.json

cd "$DIR"/demo
${FORCE_CLI} init -f --config-path "${FORCE_CONFIG_PATH}" --project-path ${DIR}
${FORCE_CLI} deploy-ckb --config-path "${FORCE_CONFIG_PATH}"

cd "$DIR"/eth-contracts
npx hardhat run scripts/deploy.js --network geth > "${FORTH_ETH_CONFIG_PATH}"

# start relayer
cd "$DIR"/demo
${FORCE_CLI} init-ckb-light-contract --config-path "${FORCE_CONFIG_PATH}" -k 0 -i 1 -f 500 -c 40000 --wait
${FORCE_CLI} ckb-relay --config-path "${FORCE_CONFIG_PATH}" -k 1 --per-amount 5 > data/ckb-relayer.log 2>&1 &
${FORCE_CLI} eth-relay --config-path "${FORCE_CONFIG_PATH}" -k 5 > data/eth-relayer.log 2>&1 &
