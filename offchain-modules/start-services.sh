#!/usr/bin/env bash
# start ckb and eth relayers
# `bash start-services.sh` run relayers on docker-dev-chain
# `bash start-services.sh -n <customed-network>` run relayers on customed network

export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli
FORCE_CONFIG_PATH=~/.force-bridge

while [[ $# -gt 0 ]]
do
key="$1"
case $key in
  -n|--network)
    FORCE_NETWORK="$2"
    shift # past argument
    shift # past value
    ;;
  *)
    echo "unknown argument"
    shift # past argument
    ;;
esac
done

cd ${PROJECT_DIR}/offchain-modules

if [ "${FORCE_NETWORK}" = "" ]
then
  ${FORCE_CLI} init-ckb-light-contract -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay -k 1 --per-amount 5 > ${FORCE_CONFIG_PATH}/data/ckb-relayer.log 2>&1 &
  ${FORCE_CLI} eth-relay -k 5 > ${FORCE_CONFIG_PATH}/data/eth-relayer.log 2>&1 &
else
  ${FORCE_CLI} init-ckb-light-contract --network "${FORCE_NETWORK}" -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay --network "${FORCE_NETWORK}" -k 1 --per-amount 5 > ${FORCE_CONFIG_PATH}/data/ckb-relayer.log 2>&1 &
  ${FORCE_CLI} eth-relay --network "${FORCE_NETWORK}" -k 5 > ${FORCE_CONFIG_PATH}/data/eth-relayer.log 2>&1 &
fi
