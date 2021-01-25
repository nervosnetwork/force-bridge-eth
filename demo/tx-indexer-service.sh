#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace
export RUST_BACKTRACE=1
export RUST_LOG=info,force=debug

# project root directory
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
FORCE_CLI=${PROJECT_DIR}/offchain-modules/target/debug/force-eth-cli

export FORCE_CONFIG_PATH=~/.force-bridge/config.toml

OFFCHAIN="$PROJECT_DIR"/offchain-modules

CKB_URL=http://127.0.0.1:8114
INDEXER_URL=http://127.0.0.1:8116
HEADER_RELAY_PRIVKEY=1
CKB_MINT_PRIVKY=2
ETH_UNLOCK_PRIVKEY=2
SQL_PATH="$PROJECT_DIR"/offchain-modules/lib/src/dapp/db/source/
DB_NAME=forcedb
DB_PATH=mysql://root:root@127.0.0.1:3306/${DB_NAME}

MYSQL_NAME=test_mysql

approve_token(){
   ${FORCE_CLI} approve --config-path "${FORCE_CONFIG_PATH}" -k 0 --erc20-addr "${TOKEN_ADDRESS}" --wait
}

start_mysql() {

    docker run -e MYSQL_ROOT_PASSWORD=root -p 3306:3306 --name ${MYSQL_NAME} -d mysql:5.7
    docker exec ${MYSQL_NAME} bash -c "echo -e '[mysqld]\nskip-grant-tables' > /etc/mysql/conf.d/my.cnf"
    docker restart ${MYSQL_NAME}
    sleep 2
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
  pm2 start --name force-server "${FORCE_CLI} dapp server  --ckb-private-key-path ${CKB_MINT_PRIVKY}  --listen-url 0.0.0.0:3003 --db-path ${DB_PATH}"
}

start_tx_relay(){
  cd ${OFFCHAIN}
  pm2 start --name ckb-indexer "${FORCE_CLI} dapp ckb-indexer --db-path ${DB_PATH} --ckb-rpc-url ${CKB_URL} --ckb-indexer-url ${INDEXER_URL}"
  pm2 start --name eth-indexer "${FORCE_CLI} dapp eth-indexer --db-path ${DB_PATH} --ckb-indexer-url ${INDEXER_URL}"
  pm2 start --name ckb-tx-relayer "${FORCE_CLI} dapp ckb-tx-relayer --db-path ${DB_PATH} -k ${ETH_UNLOCK_PRIVKEY}"
  pm2 start --name eth-tx-relayer "${FORCE_CLI} dapp eth-tx-relayer --db-path ${DB_PATH} -p ${CKB_MINT_PRIVKY} "
}

start_header_relay(){
  pm2 start --name ckb-header-relay "${FORCE_CLI} ckb-relay -k ${HEADER_RELAY_PRIVKEY} --per-amount 10  --max-tx-count 10 --mutlisig-privkeys  0"
  pm2 start --name eth-header-relay "${FORCE_CLI} eth-relay -k ${HEADER_RELAY_PRIVKEY} --multisig-privkeys 1"
}


stress_test(){
    cd ${PROJECT_DIR}/demo && rm -rf dex-crosschain-bot
    git clone https://github.com/fpChan/dex-crosschain-bot && cd dex-crosschain-bot  && git checkout main && yarn
    yarn send && rm -rf dex-crosschain-bot
}

#stop_mysql
start_mysql
sleep 30
#stop_service
#start_header_relay
sleep 60
start_server
start_tx_relay

stress_test
