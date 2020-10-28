#!/bin/sh

MINER=0x627306090abab3a6e1400e9345bc60c78a8bef57

geth init ./geth-genesis.json --datadir=/tmp/geth

geth --nousb --datadir=/tmp/geth --port 4321 --networkid 1234 --rpc --rpcport 8545 --rpcaddr 0.0.0.0  --rpcapi "eth,net,web3,personal,miner" --gasprice 0 --etherbase $MINER --mine --miner.threads=1 --nodiscover

/bin/sh "$@"
