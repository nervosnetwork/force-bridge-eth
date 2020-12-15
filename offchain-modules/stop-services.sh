#!/usr/bin/env bash

set -o errexit
set -o xtrace

ps aux | grep 'force-eth-cli ckb-relay' | grep -v grep | awk '{print $2}' | xargs kill -9
ps aux | grep 'force-eth-cli eth-relay' | grep -v grep | awk '{print $2}' | xargs kill -9
