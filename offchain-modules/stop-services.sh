#!/usr/bin/env bash

set -o errexit
set -o xtrace

ps aux | grep 'force-eth-cli ckb-relay' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'force-eth-cli eth-relay' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'dapp eth-indexer' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'dapp ckb-indexer' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'dapp ckb-tx-relayer' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'dapp eth-tx-relayer' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'dapp server' | grep -v grep | awk '{print $2}' | xargs kill -9
