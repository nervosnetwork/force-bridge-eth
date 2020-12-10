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
CKB_CLI=${PROJECT_DIR}/offchain-modules/vendor/ckb-cli
PRIVKEY_PATH=${PROJECT_DIR}/offchain-modules/privkeys
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

if [ "${FORCE_NETWORK}" = "" ]
then
  ${FORCE_CLI} init-ckb-light-contract -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay -k 1 --per-amount 5 > ${FORCE_LOG_PATH}/ckb-relayer.log 2>&1 &
#  echo "" |  ${CKB_CLI} account import --privkey-path ${PRIVKEY_PATH}/ckb1
#  echo "" |  ${CKB_CLI} account import --privkey-path ${PRIVKEY_PATH}/ckb2
  ${CKB_CLI} tx build-multisig-address --sighash-address ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37 --threshold 2 > mutisig.log
  lock_args=`cat mutisig.log| awk '{print $2}'| awk 'NR==1' `
  echo $lock_args
  address=`cat mutisig.log| awk '{print $2}'| awk 'NR==4' `
  echo $address
  echo "" |  ${CKB_CLI} wallet transfer --capacity 9000000000 --to-address $address  --from-account ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk --tx-fee 0.1
  sleep 20
  ${CKB_CLI} wallet get-capacity --address  $address
  ${FORCE_CLI} eth-relay -k 1 --multisig-args  $lock_args > ${FORCE_LOG_PATH}/eth-relayer.log 2>&1 &
else
  ${FORCE_CLI} init-ckb-light-contract --network "${FORCE_NETWORK}" -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay --network "${FORCE_NETWORK}" -k 1 --per-amount 5 > ${FORCE_LOG_PATH}/ckb-relayer.log 2>&1 &
  ${FORCE_CLI} eth-relay --network "${FORCE_NETWORK}" -k 1 > ${FORCE_LOG_PATH}/eth-relayer.log 2>&1 &
fi
