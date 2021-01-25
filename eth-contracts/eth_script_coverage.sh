#!/usr/bin/env bash

if [ -z "$CODECOV_TOKEN" ]
then
  echo "CODECOV_TOKEN not set, check https://app.codecov.io/gh/nervosnetwork/force-bridge-eth/settings"
  exit 1
fi

npx hardhat coverage
bash <(curl -s https://codecov.io/bash) -f ./coverage.json
