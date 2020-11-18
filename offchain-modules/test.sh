#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit

# Executes cleanup function at script exit.
trap cleanup EXIT

cleanup() {
    # Kill the ganache instance that we started (if we started one and if it's still running).
    if [ -n "$ganache_pid" ] && ps -p $ganache_pid > /dev/null; then
        kill -9 $ganache_pid
    fi
}

ganache_port=9545

start_ganache() {
    # We define 10 accounts with balance 1M ether, needed for high-value tests.
    local accounts=(
        --account="0x49740e7b29259e7c2b693f365a9fd581cef75d1e346c8dff89ec037cdfd9f89d,1000000000000000000000000"
        --account="0x84723abba0388d7c932deff3a02d5f8dad3a2696a7a2cee8cc42f684fcbb6f3e,1000000000000000000000000"
        --account="0xd1046c54de312b30527e06882515901d919c35010cb756b2c5f08fb608713d4f,1000000000000000000000000"
        --account="0x39caceb96e2401894785ef41d9c903a1d3250735171b1851c67d77d92ebdb0d7,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501204,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501205,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501206,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501207,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501208,1000000000000000000000000"
        --account="0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501209,1000000000000000000000000"
    )
    ganache-cli --gasLimit 0xfffffffffff --port "$ganache_port" "${accounts[@]}" > /dev/null &
    ganache_pid=$!
    sleep 5
}

echo "Starting our own ganache instance"
start_ganache

truffle version
cd ../eth-contracts && truffle migrate --reset
cd ../offchain-modules

ETH_CONTRACT_ADDRESS=0x963C9Ee211373B902402467B58B407d2065dA671
TOKEN_ADDRESS=0xEaddfCa0A3C33cd4e7A5F56bCF1cE31944dD0D0d
LOCK_TOKEN_PATH=/tmp/lock_token.log
LOCK_ETH_PATH=/tmp/lock_eth.log

target/debug/force-eth-cli dev-init -f --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --eth-token-address "${TOKEN_ADDRESS}"
target/debug/force-eth-cli approve --from "${ETH_CONTRACT_ADDRESS}" --to "${TOKEN_ADDRESS}"

target/debug/force-eth-cli lock-token  --to "${ETH_CONTRACT_ADDRESS}" --token  "${TOKEN_ADDRESS}" --amount 100 --bridge-fee 10 --sudt-extra-data sudt_extra_data > "${LOCK_TOKEN_PATH}"
lock_token_hash=`cat "${LOCK_TOKEN_PATH}"| awk '{print $5}'`
echo "${lock_token_hash}"
target/debug/force-eth-cli mint --hash "${lock_token_hash}" --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --cell depend_on_eth_relay

target/debug/force-eth-cli lock-eth --to "${ETH_CONTRACT_ADDRESS}" --amount 10000 --bridge-fee 1000  --sudt-extra-data sudt_extra_data > "${LOCK_ETH_PATH}"
lock_eth_hash=`cat "${LOCK_ETH_PATH}"| awk '{print $4}'`
echo "${lock_eth_hash}"
target/debug/force-eth-cli mint --hash "${lock_eth_hash}" --eth-contract-address "${ETH_CONTRACT_ADDRESS}" --cell depend_on_eth_relay

#target/debug/force-eth-cli ckb-relay --from 0x627306090abaB3A6e1400e9345bC60c78a8BEf57 --to 0xeec918d74c746167564401103096D45BbD494B74  --eth-rpc-url http://172.18.0.51:8545 --private-key-path cli/privkeys/key
# ckb dev_init : deploy ckb contract and prepare
#RUST_LOG=info ./target/debug/force-eth-cli dev-init  --token 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1  --lock-contract-addr 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2 -f

#RUST_LOG=info  ./target/debug/force-eth-cli  query-sudt-blance --addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37 --token-addr 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1 --lock-contract-addr 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2

#RUST_LOG=info ./target/debug/force-eth-cli burn --burn-amount 9000 --receive-addr 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1 --token-addr 0xCa5e43317e18c98b83aB1406818310A683Dcd9D1   --lock-contract-addr 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2 --unlock-fee 10 --private-key-path cli/privkeys/ckb_key

#./target/debug/force-eth-cli  generate-ckb-proof --tx-hash 0395143a90bf9a1ac669c146211724e33797cb36ff0bec0048c17d2d9a11592d

#RUST_LOG=info ./target/debug/force-eth-cli transfer-from-ckb --burn-amount 5 --unlock-fee 1 --ckb-privkey-path cli/privkeys/ckb_key --eth-privkey-path ~/account/sk  --eth-rpc-url http://172.18.0.51:8545 --receive-addr 0x403A53A7Dfa7a4AB022e53FeFf11232b3140407d   --token-addr 0xE9837e999934E2f61D4148CeEEb1Be3cc00Cb37c --lock-contract-addr 0x30F58CEEEC7Ace3fC2f938F13dD2C0c69B4A19b0 --light-client-addr 0x3C049b35392D20e5C1E7bB6B32E26f07093A6923
