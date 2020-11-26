#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
DATA_DIR="${DIR}"/demo/data
mkdir -p "${DATA_DIR}"
FORCE_CLI="${DIR}"/demo/bin/force-eth-cli
FORTH_ETH_CONFIG_PATH="${DATA_DIR}"/force-eth-config.json
ETH_BRIDGE_CELL_CONFIG_PATH="${DATA_DIR}"/eth-bridge-cell-config.json
TOKEN_BRIDGE_CELL_CONFIG_PATH="${DATA_DIR}"/token-bridge-cell-config.json
LOCK_TOKEN_PATH="${DATA_DIR}"/lock_token.log
LOCK_ETH_PATH="${DATA_DIR}"/lock_eth.log

cd "$DIR"/eth-contracts
export FORCE_CONFIG_PATH="${DIR}"/demo/.force-bridge-cli-config.toml
TOKEN_ADDRESS=$(tail -1 ${FORTH_ETH_CONFIG_PATH} | jq -r .erc20)
ETH_ADDRESS="0x0000000000000000000000000000000000000000"
RECIPIENT_ADDR="ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4"
bridge_fee=2

cd "$DIR"/demo

# eth crosschain
${FORCE_CLI} create-bridge-cell --eth-token-address "${ETH_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${ETH_BRIDGE_CELL_CONFIG_PATH}"
eth_bridge_cell_outpoint=$(cat "${ETH_BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
${FORCE_CLI} lock-eth --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${eth_bridge_cell_outpoint}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_ETH_PATH}"
lock_eth_hash=`cat "${LOCK_ETH_PATH}"| awk '{print $4}'`

# token crosschain
${FORCE_CLI} approve --erc20-addr "${TOKEN_ADDRESS}"
${FORCE_CLI} create-bridge-cell --eth-token-address "${TOKEN_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${TOKEN_BRIDGE_CELL_CONFIG_PATH}"
token_bridge_cell_outpoint=$(cat "${TOKEN_BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
${FORCE_CLI} lock-token --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${token_bridge_cell_outpoint}" --token  "${TOKEN_ADDRESS}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_TOKEN_PATH}"
lock_token_hash=`cat "${LOCK_TOKEN_PATH}"| awk '{print $5}'`

# eth crosschain
${FORCE_CLI} mint --hash "${lock_eth_hash}"
${FORCE_CLI} query-sudt-blance --addr ${RECIPIENT_ADDR} --token-addr "${ETH_ADDRESS}"
${FORCE_CLI} transfer-from-ckb --ckb-privkey-path privkeys/ckb_key_recipient --burn-amount 2 --unlock-fee 1 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr ${ETH_ADDRESS} --wait

# token crosschain
${FORCE_CLI} mint --hash "${lock_token_hash}"
${FORCE_CLI} query-sudt-blance --addr ${RECIPIENT_ADDR} --token-addr "${TOKEN_ADDRESS}"
${FORCE_CLI} transfer-from-ckb --ckb-privkey-path privkeys/ckb_key_recipient --burn-amount 2 --unlock-fee 1 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d --wait --token-addr "${TOKEN_ADDRESS}"
