#!/usr/bin/env bash
# start ckb and eth relayers
# `bash start-services.sh` run relayers on docker-dev-chain
# `bash start-services.sh -n <customed-network>` run relayers on customed network

set -o errexit
set -o xtrace

export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli
FORCE_LOG_PATH=~/.force-bridge/logs
mkdir -p "${FORCE_LOG_PATH}"

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


TELEGRAM_TOKEN=1449859422:AAG3UQvRnEsr3wv7_UeQUehFkkZpdwP_x-Y
TELEGRAM_CHAT_ID=-416140042
TELEGRAM_URL=https://api.telegram.org/bot${TELEGRAM_TOKEN}/sendMessage\?chat_id\=${TELEGRAM_CHAT_ID}\&text\=


if [ "${FORCE_NETWORK}" = "" ]
then
  ${FORCE_CLI} init-ckb-light-contract -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay -k 1 --per-amount 5 > ${FORCE_LOG_PATH}/ckb-relayer.log 2>&1 &
  ${FORCE_CLI} eth-relay -k 3 > ${FORCE_LOG_PATH}/eth-relayer.log 2>&1 &
  sleep 40
  ${FORCE_CLI} relayer-monitor --alarm-url "${TELEGRAM_URL}" --ckb-conservator gif fpchan --eth-conservator gif fpchan > ${FORCE_LOG_PATH}/relayer_monitor.log 2>&1 &
else
  ${FORCE_CLI} init-ckb-light-contract --network "${FORCE_NETWORK}" -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay --network "${FORCE_NETWORK}" -k 1 --per-amount 5 > ${FORCE_LOG_PATH}/ckb-relayer.log 2>&1 &
  ${FORCE_CLI} eth-relay --network "${FORCE_NETWORK}" -k 3 > ${FORCE_LOG_PATH}/eth-relayer.log 2>&1 &
  sleep 40
  ${FORCE_CLI} relayer-monitor --network "${FORCE_NETWORK}" --alarm-url "${TELEGRAM_URL}" --ckb-conservator gif fpchan --eth-conservator gif fpchan > ${FORCE_LOG_PATH}/relayer_monitor.log 2>&1 &
fi
