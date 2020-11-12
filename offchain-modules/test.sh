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
RUST_LOG="debug" target/debug/force-eth-cli lock-eth --to 0Ef621E386a0F30CBFa3AbDC6363bc4219627162 --amount 10000 --bridge-fee 1000  --sudt-extra-data sudt_extra_data
# ckb-relay example
# target/debug/force-eth-cli ckb-relay --from 0x627306090abaB3A6e1400e9345bC60c78a8BEf57 --to 0xeec918d74c746167564401103096D45BbD494B74  --eth-rpc-url http://172.18.0.51:8545 --private-key-path cli/privkeys/key
