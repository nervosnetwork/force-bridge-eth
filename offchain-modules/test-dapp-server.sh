#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# install pm2
#sudo npm i pm2 -g

# project root directory
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli

export FORCE_CONFIG_PATH=~/.force-bridge/config.toml

OFFCHAIN="$PROJECT_DIR"/offchain-modules

HEADER_RELAY_PRIVKEY=1
CKB_MINT_PRIVKY=2
API_SERVER_PRIVKEY="4 5"
ETH_UNLOCK_PRIVKEY=2
SQL_PATH="$PROJECT_DIR"/offchain-modules/lib/src/dapp/db/source/
DB_NAME=forcedb
DB_PATH=mysql://root:root@127.0.0.1:3306/${DB_NAME}

MYSQL_NAME=test_mysql


FORCE_LOG_PATH=~/.force-bridge/logs
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

stop_mysql(){
    docker stop `docker ps | grep ${MYSQL_NAME} | awk '{print $1}'`
    docker rm `docker ps -a | grep ${MYSQL_NAME} | awk '{print $1}'`
}

services=("ckb-indexer" "eth-indexer" "force-server" "ckb-tx-relayer" "eth-tx-relayer")
stop_service() {
  all=*
  for service in "${services[@]}"
  do
    pm2 stop $service && pm2 delete $service && rm -f ~/.pm2/logs/$service$all
  done
}

start_server(){
  cd ${OFFCHAIN}
  ${FORCE_CLI} dapp server  --server-private-key-path ${API_SERVER_PRIVKEY}  --mint-private-key-path ${CKB_MINT_PRIVKY} --listen-url 0.0.0.0:3003 --db-path ${DB_PATH} > ${FORCE_LOG_PATH}/force-server.log 2>&1 &
}

start_tx_relay(){
  cd ${OFFCHAIN}
  ${FORCE_CLI} dapp ckb-indexer --db-path ${DB_PATH} > /home/runner/work/force-bridge-eth/ckb-indexer.log 2>&1 &
  ${FORCE_CLI} dapp eth-indexer --db-path ${DB_PATH} > /home/runner/work/force-bridge-eth/eth-indexer.log 2>&1 &
  ${FORCE_CLI} dapp ckb-tx-relayer --db-path ${DB_PATH} -k ${ETH_UNLOCK_PRIVKEY} > /home/runner/work/force-bridge-eth/ckb-tx-relayer.log 2>&1 &
  ${FORCE_CLI} dapp eth-tx-relayer --db-path ${DB_PATH} -p ${CKB_MINT_PRIVKY} > /home/runner/work/force-bridge-eth/eth-tx-relayer.log 2>&1 &
}

start_header_indexer_relay() {
  cd ${OFFCHAIN}
  ${FORCE_CLI} dapp ckb-header-indexer > ${FORCE_LOG_PATH}/ckb-header-indexer.log 2>&1 &
  ${FORCE_CLI} dapp eth-header-indexer > ${FORCE_LOG_PATH}/eth-header-indexer.log 2>&1 &
}

#start_server(){
#  cd ${OFFCHAIN}
#  pm2 start --name force-server "${FORCE_CLI} dapp server  --ckb-private-key-path ${CKB_MINT_PRIVKY}  --listen-url 0.0.0.0:3003 --db-path ${DB_PATH}"
#}
#
#start_tx_relay(){
#  cd ${OFFCHAIN}
#  pm2 start --name ckb-indexer "${FORCE_CLI} dapp ckb-indexer --db-path ${DB_PATH}"
#  pm2 start --name eth-indexer "${FORCE_CLI} dapp eth-indexer --db-path ${DB_PATH}"
#  pm2 start --name ckb-tx-relayer "${FORCE_CLI} dapp ckb-tx-relayer --db-path ${DB_PATH} -k ${ETH_UNLOCK_PRIVKEY}"
#  pm2 start --name eth-tx-relayer "${FORCE_CLI} dapp eth-tx-relayer --db-path ${DB_PATH} -p ${CKB_MINT_PRIVKY} "
#}

start_header_relay(){
  pm2 start --name ckb-header-relay "${FORCE_CLI} ckb-relay -k ${HEADER_RELAY_PRIVKEY} --per-amount 10  --max-tx-count 10 --mutlisig-privkeys  0"
  pm2 start --name eth-header-relay "${FORCE_CLI} eth-relay -k ${HEADER_RELAY_PRIVKEY} --multisig-privkeys 1"
}


stress_test(){
    cd ${PROJECT_DIR}/offchain-modules/stress_test/ && yarn && yarn send
}

#stop_mysql
#start_mysql
#sleep 10
#stop_service
#start_header_relay
start_server
sleep 3
start_header_indexer_relay
sleep 10
start_tx_relay


stress_test
