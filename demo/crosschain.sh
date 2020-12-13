#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# project root directory
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
DATA_DIR="${PROJECT_DIR}"/demo/data
mkdir -p "${DATA_DIR}"
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli
FORTH_ETH_CONFIG_PATH=~/.force-bridge/erc20-contracts.json
ETH_BRIDGE_CELL_CONFIG_PATH="${DATA_DIR}"/eth-bridge-cell-config.json
TOKEN_BRIDGE_CELL_CONFIG_PATH="${DATA_DIR}"/token-bridge-cell-config.json
LOCK_TOKEN_PATH="${DATA_DIR}"/lock_token.log
LOCK_ETH_PATH="${DATA_DIR}"/lock_eth.log

export FORCE_CONFIG_PATH=~/.force-bridge/config.toml
TOKEN_ADDRESS=$(tail -1 ${FORTH_ETH_CONFIG_PATH} | jq -r .daiContractAddr)
ETH_ADDRESS="0x0000000000000000000000000000000000000000"
RECIPIENT_ADDR="ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4"
bridge_fee=8

cd "$PROJECT_DIR"/offchain-modules

# eth crosschain
#${FORCE_CLI} create-bridge-cell --config-path "${FORCE_CONFIG_PATH}" -k 0 --eth-token-address "${ETH_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${ETH_BRIDGE_CELL_CONFIG_PATH}"
#bridge_cell_outpoint=$(cat "${ETH_BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
#${FORCE_CLI} lock-eth --config-path "${FORCE_CONFIG_PATH}" -k 0 --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${bridge_cell_outpoint}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_ETH_PATH}"
#lock_eth_hash=`cat "${LOCK_ETH_PATH}"| awk '{print $4}'`
#http 127.0.0.1:3030/relay_eth_to_ckb_proof <<< "{\"eth_lock_tx_hash\": \"${lock_eth_hash}\"}"
${FORCE_CLI} burn --config-path "${FORCE_CONFIG_PATH}" --private-key-path 0 --burn-amount 2 --unlock-fee 1 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr ${ETH_ADDRESS}

