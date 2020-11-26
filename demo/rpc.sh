#!/usr/bin/env bash
# You need to install [httpie](https://github.com/httpie/httpie) first

# Exit script as soon as a command fails.
set -o errexit
set -o xtrace

URL=127.0.0.1:3030

http "${URL}"/settings

http "${URL}"/get_or_create_bridge_cell <<< '''
{
  "recipient_address": "ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4",
  "eth_token_address": "0x0000000000000000000000000000000000000000",
  "bridge_fee": "0x0"
}
'''

http "${URL}"/lock <<< '''
{
  "token_address": "0x3E35617a629EEaD5a6767dC69D238831a7Bc391c",
  "amount": "0x100",
  "bridge_fee": "0x1",
  "ckb_recipient_address": "ckt1qyqdtmf8yy3agyarxpwre60ydrl8082n5hysd4nn86",
  "replay_resist_outpoint": "bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000",
  "sudt_extra_data": "bbc62a3943831e918f864a0306bc794b9e2724afc141b69d47e070f1f5686d4200000000",
  "gas_price": "0x1",
  "nonce": "0x1"
}
'''

http "${URL}"/burn <<< '''
{
  "from_lockscript_addr": "ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4",
  "tx_fee": "0.01",
  "unlock_fee": "0x0",
  "amount": "0x0",
  "token_address": "0x0000000000000000000000000000000000000000",
  "recipient_address": "0x728060aB319E06AC6b5d5c0C74009F7D1f5730DB"
}
'''

http "${URL}"/get_sudt_balance <<< '''
{
  "address": "ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4",
  "token_address": "0x0000000000000000000000000000000000000000"
}
'''
