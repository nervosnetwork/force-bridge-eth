#!/bin/sh

MINER=0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2

geth init ./geth-genesis.json --datadir=/tmp/geth

geth --nousb --datadir=/tmp/geth --port 4321 --networkid 1234 --rpc --rpcport 8545 --rpcaddr 0.0.0.0  --rpcapi "eth,net,web3,personal,miner" --gasprice 0 --etherbase $MINER --mine --miner.threads=1 --nodiscover

/bin/sh "$@"
