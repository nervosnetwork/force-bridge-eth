#!/usr/bin/env bash
# start ckb and eth relayers
# `bash start-services.sh` run relayers on docker-dev-chain
# `bash start-services.sh -n <customed-network>` run relayers on customed network

set -o errexit
set -o xtrace

export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug
export FORCE_CONFIG_PATH=~/.force-bridge/config.toml

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli
SIGN_CLI=${PROJECT_DIR}/offchain-modules/target/debug/sign-server
FORCE_LOG_PATH=~/.force-bridge/logs
SQL_PATH="$PROJECT_DIR"/offchain-modules/lib/src/dapp/db/source/
DB_NAME=forcedb
DB_PATH=mysql://root:root@127.0.0.1:3306/${DB_NAME}
MYSQL_NAME=test_mysql
mkdir -p "${FORCE_LOG_PATH}"

start_mysql() {

    docker run -e MYSQL_ROOT_PASSWORD=root -p 3306:3306 --name ${MYSQL_NAME} -d mysql:5.7
    docker exec ${MYSQL_NAME} bash -c "echo -e '[mysqld]\nskip-grant-tables' > /etc/mysql/conf.d/my.cnf"
    docker restart ${MYSQL_NAME}
    sleep 8
#    docker exec ${MYSQL_NAME} mysql --user root --password=root -e "drop database ${DB_NAME};"
    docker exec ${MYSQL_NAME} mysql --user root --password=root -e "create database ${DB_NAME}; use ${DB_NAME}; show tables;"
    files=$(ls $SQL_PATH)
    for filename in $files
    do
      if [ "${filename##*.}" = "sql" ]; then
        docker cp $SQL_PATH$filename ${MYSQL_NAME}:/tmp/$filename
        docker exec ${MYSQL_NAME} mysql --user root --password=root -e "use ${DB_NAME};source /tmp/${filename};"
        sleep 1
      fi
    done
    docker exec ${MYSQL_NAME} mysql --user root --password=root -e "show databases;"
    echo "***** start mysql successfully *****"
}

start_indexer() {
  cd ${PROJECT_DIR}/offchain-modules
  ${FORCE_CLI} dapp ckb-indexer --db-path ${DB_PATH} > ${FORCE_LOG_PATH}/ckb-indexer.log 2>&1 &
  ${FORCE_CLI} dapp eth-indexer --db-path ${DB_PATH} > ${FORCE_LOG_PATH}/eth-indexer.log 2>&1 &
}

start_sign_server() {
  cell_script=`cat ${FORCE_CONFIG_PATH}|grep "cell_script" | awk '{print $3}' | sed 's/\"//g'`
  echo ${cell_script}
  cd ${PROJECT_DIR}/offchain-modules/sign-server
  cp ${PROJECT_DIR}/offchain-modules/target/debug/sign-server .
  ${PROJECT_DIR}/offchain-modules/sign-server/sign-server run --cell-script ${cell_script}> ${FORCE_LOG_PATH}/sign-server-0.log 2>&1 &
  ${PROJECT_DIR}/offchain-modules/sign-server/sign-server run --config-path conf_1/config.toml --listen-url 0.0.0.0:3032 --ckb-key-path conf_1/ckb_key --eth-key-path conf_1/eth_key --cell-script ${cell_script}> ${FORCE_LOG_PATH}/sign-server-1.log 2>&1 &
  ${PROJECT_DIR}/offchain-modules/sign-server/sign-server run --config-path conf_2/config.toml --listen-url 0.0.0.0:3033 --ckb-key-path conf_2/ckb_key --eth-key-path conf_2/eth_key --cell-script ${cell_script}> ${FORCE_LOG_PATH}/sign-server-2.log 2>&1 &
  sleep 5
}

start_sign_server

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
#  ${FORCE_CLI} init-ckb-light-contract -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay -k 1 --per-amount 5  --max-tx-count 10 --hosts http://127.0.0.1:3031 http://127.0.0.1:3032 http://127.0.0.1:3033 > ${FORCE_LOG_PATH}/ckb-relayer.log 2>&1 &
#  ${FORCE_CLI} init-multi-sign-address -k 1 --multi-address  ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
  ${FORCE_CLI} eth-relay -k 1 --confirm 5 --delay 30 > ${FORCE_LOG_PATH}/eth-relayer.log 2>&1 &
else
#  ${FORCE_CLI} init-ckb-light-contract --network "${FORCE_NETWORK}" -k 0 -f 500 -c 40000 --wait
  ${FORCE_CLI} ckb-relay --network "${FORCE_NETWORK}" -k 1 --per-amount 5  --max-tx-count 10 -mutlisig-privkeys  0 > ${FORCE_LOG_PATH}/ckb-relayer.log 2>&1 &
#  ${FORCE_CLI} init-multi-sign-address --network "${FORCE_NETWORK}" -k 1 --multi-address  ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
  ${FORCE_CLI} eth-relay --network "${FORCE_NETWORK}" -k 1 > ${FORCE_LOG_PATH}/eth-relayer.log 2>&1 &
fi

#sleep 10
#cat ${FORCE_LOG_PATH}/eth-relayer.log
##echo ${eth_relayer_log}
#cat ${FORCE_LOG_PATH}/sign-server.log
start_mysql
sleep 10
start_indexer
