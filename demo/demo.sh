#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# project root directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"

FORTH_ETH_CONFIG_PATH=/tmp/force-eth-config.json
BRIDGE_CELL_CONFIG_PATH=/tmp/bridge-cell-config.json
LOCK_TOKEN_PATH=/tmp/lock_token.log
LOCK_ETH_PATH=/tmp/lock_eth.log

cd "$DIR"/offchain-modules
target/debug/force-eth-cli dev-init -f

cd "$DIR"/eth-contracts
npx hardhat run scripts/deploy.js --network geth > "${FORTH_ETH_CONFIG_PATH}"
ETH_CONTRACT_ADDRESS=$(tail -1 ${FORTH_ETH_CONFIG_PATH} | jq -r .tokenLocker)
TOKEN_ADDRESS=$(tail -1 ${FORTH_ETH_CONFIG_PATH} | jq -r .erc20)
ETH_ADDRESS="0x0000000000000000000000000000000000000000"
RECIPIENT_ADDR="ckt1qyqt6tdevvfgllmzpkz8v9dz3zv082v7ttxqly2899"
bridge_fee=2

cd "$DIR"/offchain-modules

target/debug/force-eth-cli approve --from "${ETH_CONTRACT_ADDRESS}" --to "${TOKEN_ADDRESS}" --wait
target/debug/force-eth-cli create-bridge-cell --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --eth-token-address "${TOKEN_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${BRIDGE_CELL_CONFIG_PATH}"
bridge_cell_outpoint=$(cat "${BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
target/debug/force-eth-cli lock-token --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${bridge_cell_outpoint}" --to "${ETH_CONTRACT_ADDRESS}" --token  "${TOKEN_ADDRESS}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_TOKEN_PATH}"
lock_token_hash=`cat "${LOCK_TOKEN_PATH}"| awk '{print $5}'`
echo "${lock_token_hash}"
target/debug/force-eth-cli mint --hash "${lock_token_hash}" --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --cell depend_on_eth_relay

#target/debug/force-eth-cli query-sudt-blance --addr ${RECIPIENT_ADDR} --token-addr "${TOKEN_ADDRESS}" --lock-contract-addr "${ETH_CONTRACT_ADDRESS}"
#target/debug/force-eth-cli transfer-from-ckb --burn-amount 5 --unlock-fee 1  --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr $ERC20DeployAddr --lock-contract-addr $TokenLockerDepolyAddr --light-client-addr $CKBChainDeployAddr

target/debug/force-eth-cli create-bridge-cell --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --eth-token-address "${ETH_ADDRESS}" --recipient-address "${RECIPIENT_ADDR}" --bridge-fee "${bridge_fee}" > "${BRIDGE_CELL_CONFIG_PATH}"
bridge_cell_outpoint=$(cat "${BRIDGE_CELL_CONFIG_PATH}" | jq -r .outpoint)
target/debug/force-eth-cli lock-eth --ckb-recipient-address "${RECIPIENT_ADDR}" --replay-resist-outpoint "${bridge_cell_outpoint}" --to "${ETH_CONTRACT_ADDRESS}" --amount 100 --bridge-fee "${bridge_fee}" --sudt-extra-data sudt_extra_data --wait > "${LOCK_ETH_PATH}"
lock_eth_hash=`cat "${LOCK_ETH_PATH}"| awk '{print $4}'`
echo "${lock_eth_hash}"
target/debug/force-eth-cli mint --hash "${lock_eth_hash}" --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --cell depend_on_eth_relay


#
#target/debug/force-eth-cli lock-eth --to "${ETH_CONTRACT_ADDRESS}" --amount 10000 --bridge-fee 1000  --sudt-extra-data sudt_extra_data > "${LOCK_ETH_PATH}"
#lock_eth_hash=`cat "${LOCK_ETH_PATH}"| awk '{print $4}'`
#echo "${lock_eth_hash}"
#target/debug/force-eth-cli mint --hash "${lock_eth_hash}" --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --cell depend_on_eth_relay
#
##target/debug/force-eth-cli ckb-relay --from 0x627306090abaB3A6e1400e9345bC60c78a8BEf57 --to 0xeec918d74c746167564401103096D45BbD494B74  --eth-rpc-url http://172.18.0.51:8545 --private-key-path cli/privkeys/key
## ckb dev_init : deploy ckb contract and prepare
##RUST_LOG=info ./target/debug/force-eth-cli dev-init  --token 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1  --lock-contract-addr 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2 -f
#
##RUST_LOG=info  ./target/debug/force-eth-cli  query-sudt-blance --addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37 --token-addr 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1 --lock-contract-addr 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2
#
##RUST_LOG=info ./target/debug/force-eth-cli burn --burn-amount 9000 --receive-addr 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1 --token-addr 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1   --lock-contract-addr 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2 --unlock-fee 10 --private-key-path cli/privkeys/ckb_key
#
##./target/debug/force-eth-cli  generate-ckb-proof --tx-hash 0395143a90bf9a1ac669c146211724e33797cb36ff0bec0048c17d2d9a11592d

