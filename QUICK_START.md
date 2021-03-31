# Start Force Bridge Eth From Scratch Using CLI
This tutorial introduces how to set up the force bridge infrastructure and start cross-chain from scratch.

## Table of Contents
- [Build the Project](#build-the-project)
- [Configure Your Bridge](#configure-your-bridge)
- [Deploy Contracts](#deploy-contracts)
- [Start Block Header Relay](#start-block-header-relay)
- [Bridge Assets](#bridge-assets)

## Build the Project
### 1. Download
```bash
git clone https://github.com/nervosnetwork/force-bridge-eth.git
# cd force-bridge-eth project dir
git submodule update --init
```

### 2. Build
```bash
# cd force-bridge-eth project dir
make build-all
```

## Configure Your Bridge
### 1. Init config
```bash
# this command will generate default config file: ~/.force-bridge/config.toml 
make init-config
```

### 2. Customize `config.toml`

Replace the `default_network` with your own config
   
```toml
default_network = 'your-customized-network'
[networks_config.your-customized-network]
# replace with your ckb node url
ckb_rpc_url = 'http://127.0.0.1:8114'
# replace with your ckb indexer url
ckb_indexer_url = 'http://127.0.0.1:8116'
ckb_private_keys = [
    'your-private-key-to-deploy-ckb-contracts',
    'your-private-key-to-relay-ethereum-block-headers',
    'your-private-key-to-mint/burn-sUDT-on-ckb',
]
# replace with your ethereum url
ethereum_rpc_url = 'http://127.0.0.1:8545'
ethereum_private_keys = [
    'your-private-key-to-deploy-ethereum-contracts',
    'your-private-key-to-relay-ckb-block-headers',
    'your-private-key-to-lock/unlock-tokens-on-ethereum',
]
```

## Deploy Contracts
### 1. Deploy ckb-contracts
```bash
make deploy-ckb
```

> If you want set up bridge between dev-chains, you should run `make deploy-ckb-sudt` instead, which will deploy sudt-script on ckb-dev-chain and you can skip the second step below.

### 2. Add sUDT config

Add sUDT info to `config.toml` according to your network type:
  - ref：https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-system-script-list/0024-ckb-system-script-list.md#simple-udt

```toml
# if you use ckb aggron testnet, here is the example:
[deployed_contracts.sudt]
code_hash = "c5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4"
hash_type = 1

  [deployed_contracts.sudt.outpoint]
  tx_hash = "e12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769"
  index = 0
  dep_type = 0 
```

### 3. Init eth-light-client-multi-address on Ckb

Deploy the multi-sig-lockscript with your multi-addresses to manage the eth-light-client cell.

```bash
export FORCE_CLI=./offchain-modules/target/debug/force-eth-cli

# use -k to get the private key to sign the tx, you can assign path_to_private_key_file or index_of_ckb_private_keys_in_config_file
# use --multi-address to set the multi-sig-addresses, please replace with the first two addresses in your ckb_private_keys config
${FORCE_CLI} init-multi-sign-address -k 0 --multi-address  ckt1qyqyph8v9mclls35p6snlaxajeca97tc062sa5gahk ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
```

### 4. Deploy eth-contracts
```bash
make deploy-eth
```

> If you want set up bridge between dev-chains, you can run `make deploy-erc-20` additional, which will deploy erc-20 contract on ethereum-dev-chain for you.

## Start Block Header Relay
Block header relay are off-chain processes to keep ckb and eth light client sync.

### 1. Start ckb-header-relay 
```bash
${FORCE_CLI} ckb-relay -k 1 --per-amount 5  --max-tx-count 10 --mutlisig-privkeys  0 
```

### 2. Start eth-header-relay
```bash
${FORCE_CLI} eth-relay -k 1 --multisig-privkeys 0 1 --confirm 5 --delay 30
```

## Bridge Assets
We use ETH as an example to show the bridge process, you may bridge any erc-20 tokens if you prefer.

### ETH -> ckETH
1.Create Bridge Cell

The bridge cell can verify cross-chain message and mint sUDT, so we create our own bridge cell using the command below:
   
```bash
# ETH token address: 0x0000000000000000000000000000000000000000
${FORCE_CLI} create-bridge-cell -k 2 -s --eth-token-address 0x0000000000000000000000000000000000000000
```

Pay attention to the command output and note the `bridge_cell_outpiont` info, which will be used in next step.

2.Lock ETH on Ethereum

```bash
${FORCE_CLI} lock-eth --config-path -k 2 --ckb-recipient-address "${YOUR_RECIPIENT_ADDR}" --replay-resist-outpoint "${bridge_cell_outpoint}" --amount 100 --bridge-fee 0 --wait
```

Pay attention to the command output and note the `lock_eth_tx_hash` info, which will be used in next step.

3.Mint ckETH on Ckb

```bash
${FORCE_CLI} mint -k 2 --hash "${lock_eth_tx_hash}"
```

You can query your ckETH balance:

```bash
${FORCE_CLI} query-sudt-blance --addr ${YOUR_RECIPIENT_ADDR} --token-addr 0x0000000000000000000000000000000000000000
```

### ckETH -> ETH
1.Burn ckETH on Ckb
```bash
${FORCE_CLI} burn -k 2 --token-addr 0x0000000000000000000000000000000000000000 --receive-addr "${YOUR_ETH_ADDRESS}" --burn-amount 100 --unlock-fee 0
```

Pay attention to the command output and note the `burn_tx_hash` info, which will be used in next step.

2.Unlock ETH on Ethereum
```bash
${FORCE_CLI} unlock -k 2 --burn-tx-hash "${burn_tx_hash}" --wait
```
