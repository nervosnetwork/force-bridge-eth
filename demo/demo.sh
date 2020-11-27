#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
trap cleanup EXIT

export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
export FORCE_CONFIG_PATH="${DIR}"/demo/.force-bridge-cli-config.toml
DATA_DIR="${DIR}"/demo/data
mkdir -p "${DATA_DIR}"
FORCE_CLI="${DIR}"/demo/bin/force-eth-cli
FORTH_ETH_CONFIG_PATH="${DATA_DIR}"/force-eth-config.json
ETH_BRIDGE_CELL_CONFIG_PATH="${DATA_DIR}"/eth-bridge-cell-config.json
TOKEN_BRIDGE_CELL_CONFIG_PATH="${DATA_DIR}"/token-bridge-cell-config.json
LOCK_TOKEN_PATH="${DATA_DIR}"/lock_token.log
LOCK_ETH_PATH="${DATA_DIR}"/lock_eth.log

cleanup() {
    # Kill the relay service && docker instance that we started (if we started one and if it's still running).
    ps aux | grep 'force-eth-cli ckb-relay' | grep -v grep | awk '{print $2}' | xargs kill -9
    ps aux | grep 'force-eth-cli eth-relay' | grep -v grep | awk '{print $2}' | xargs kill -9

    cd "$DIR"/docker
    docker-compose down
#    docker ps -a | grep tockb | awk '{print $1}' | xargs docker stop
#    docker ps -a | grep tockb | awk '{print $1}' | xargs docker rm
}

cd "$DIR"/demo
${FORCE_CLI} init -f --config-path "${FORCE_CONFIG_PATH}" --project-path ${DIR}
${FORCE_CLI} deploy-ckb --config-path "${FORCE_CONFIG_PATH}"

cd "$DIR"/eth-contracts
npx hardhat run scripts/deploy.js > "${FORTH_ETH_CONFIG_PATH}"
TOKEN_ADDRESS=$(tail -1 ${FORTH_ETH_CONFIG_PATH} | jq -r .erc20)
ETH_ADDRESS="0x0000000000000000000000000000000000000000"
RECIPIENT_ADDR="ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4"
bridge_fee=2

cd "$DIR"/demo

# start relayer
${FORCE_CLI} init-ckb-light-contract --config-path "${FORCE_CONFIG_PATH}" -k 0 -i 1 -f 500 -c 40000 --wait
${FORCE_CLI} ckb-relay --config-path "${FORCE_CONFIG_PATH}" -k 1 --per-amount 5 > data/ckb-relayer.log 2>&1 &
${FORCE_CLI} eth-relay --config-path "${FORCE_CONFIG_PATH}" -k 5 > data/eth-relayer.log 2>&1 &

# eth crosschain
${FORCE_CLI} create-bridge-cell --config-path "${FORCE_CONFIG_PATH}" -k 1 --eth-token-address "${ETH_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${ETH_BRIDGE_CELL_CONFIG_PATH}"
bridge_cell_outpoint=$(cat "${ETH_BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
${FORCE_CLI} lock-eth --config-path "${FORCE_CONFIG_PATH}" -k 0 --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${bridge_cell_outpoint}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_ETH_PATH}"
lock_eth_hash=`cat "${LOCK_ETH_PATH}"| awk '{print $4}'`
${FORCE_CLI} mint --config-path "${FORCE_CONFIG_PATH}" -k 1 --hash "${lock_eth_hash}"
${FORCE_CLI} query-sudt-blance --config-path "${FORCE_CONFIG_PATH}" --addr ${RECIPIENT_ADDR} --token-addr "${ETH_ADDRESS}"
${FORCE_CLI} transfer-from-ckb --config-path "${FORCE_CONFIG_PATH}" --ckb-privkey-path 1 --eth-privkey-path 3 --burn-amount 2 --unlock-fee 1 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr ${ETH_ADDRESS} --wait

# token crosschain
${FORCE_CLI} approve --config-path "${FORCE_CONFIG_PATH}" -k 0 --erc20-addr "${TOKEN_ADDRESS}"
${FORCE_CLI} create-bridge-cell --config-path "${FORCE_CONFIG_PATH}" -k 1 --eth-token-address "${TOKEN_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${TOKEN_BRIDGE_CELL_CONFIG_PATH}"
token_bridge_cell_outpoint=$(cat "${TOKEN_BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
${FORCE_CLI} lock-token --config-path "${FORCE_CONFIG_PATH}" -k 0 --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${token_bridge_cell_outpoint}" --token  "${TOKEN_ADDRESS}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_TOKEN_PATH}"
lock_token_hash=`cat "${LOCK_TOKEN_PATH}"| awk '{print $5}'`
${FORCE_CLI} mint --config-path "${FORCE_CONFIG_PATH}" -k 1 --hash "${lock_token_hash}"
${FORCE_CLI} query-sudt-blance --config-path "${FORCE_CONFIG_PATH}" --addr ${RECIPIENT_ADDR} --token-addr "${TOKEN_ADDRESS}"
${FORCE_CLI} transfer-from-ckb --config-path "${FORCE_CONFIG_PATH}" --ckb-privkey-path 1 --eth-privkey-path 3 --burn-amount 2 --unlock-fee 1 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d --wait --token-addr "${TOKEN_ADDRESS}"
