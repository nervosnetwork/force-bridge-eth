#!/usr/bin/env bash
# deploy contracts and scripts to ckb and ethereum

export RUST_BACKTRACE=1
export FORCE_CONFIG_PATH=~/.force-bridge-cli/config.toml
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
ETH_CONTRACTS_DIR=${PROJECT_DIR}/eth-contracts
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli

${FORCE_CLI} init -f --project-path "${PROJECT_DIR}"
${FORCE_CLI} deploy-ckb
cd "${ETH_CONTRACTS_DIR}"
npx hardhat run scripts/geth/deployAll.js



