#!/usr/bin/env bash
# deploy contracts and scripts to ckb and ethereum
# `bash deploy.sh` will deploy to default network, which is docker-dev-chain, and write deployed eth contracts address to /tmp/eth-contracts.json
# `bash deploy.sh -n <customed-network> -e <customed-eth-contracts-path>` will deploy to customed network, and write deployed eth contracts address to customed path

export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug
export FORCE_CONFIG_PATH=~/.force-bridge/config.toml

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
ETH_CONTRACTS_DIR=${PROJECT_DIR}/eth-contracts
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli

${FORCE_CLI} init --project-path "${PROJECT_DIR}"

while [[ $# -gt 0 ]]
do
key="$1"
case $key in
  -n|--network)
    export FORCE_NETWORK="$2"
    shift # past argument
    shift # past value
    ;;
  -e|--eth-contracts-json)
    ETH_CONTRACTS_JSON="$2"
    shift # past argument
    shift # past value
    ;;
  *)
    echo "unknown argument"
    shift # past argument
    ;;
esac
done

if [ "${FORCE_NETWORK}" = "" ]
then
  ${FORCE_CLI} deploy-ckb
else
  ${FORCE_CLI} deploy-ckb --network "${FORCE_NETWORK}"
fi

if [ "${ETH_CONTRACTS_JSON}" = "" ]
then
  cd ${ETH_CONTRACTS_DIR} && npx hardhat run ./scripts/deploy.js > ~/.force-bridge/erc20-contracts.json
else
  cd ${ETH_CONTRACTS_DIR} && npx hardhat run ./scripts/deploy.js > ${ETH_CONTRACTS_JSON}
fi
