#!/usr/bin/env bash

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace

URL=127.0.0.1:3030

http "${URL}"/get_or_create_bridge_cell <<< '''
{
  "recipient_address": "ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4",
  "eth_token_address": "0x0000000000000000000000000000000000000001",
  "bridge_fee": "0x0"
}
'''