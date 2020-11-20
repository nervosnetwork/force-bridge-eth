# Eth Contracts
The ethereum contracts component of `forece-bridge-eth`, mainly contains `CKBChain` and `TokenLocker`.
- `CKBChain` is a ckb light client and spv prover.
- `TokenLocker` locks erc-20 token or eth, and emits events, which can be used to mint sUDT token on CKB. In reverse, it handle bridge message(sUDT burned) from CKB, and unlock erc-20 token or eth if the message is verified.
## Quick Start
```bash
# install
$ git clone https://github.com/nervosnetwork/force-bridge-eth.git
$ cd force-bridge-eth/eth-contracts
$ npm install
$ npx hardhat test
```

## Test on Geth private chain
```bash
$ npm run test
```

## Test on Testnet
```bash
$ cp .env.example .env
# replace the private key example in the `.env` with your own private key
# make sure you have enough test ethers in your Ropsten account for delopying the contract
$ npx hardhat run scripts/testnet-ropsten/test-proveTxExist.js --network ropsten
```

## Test on Geth private chain
```bash
# install geth, please refer to https://geth.ethereum.org/docs/install-and-build/installing-geth
# after installed
$ geth init test/data/geth-genesis.json --datadir=/tmp/geth
$ geth --datadir=/tmp/geth --port 4321 --networkid 1234 --rpc --rpcport 8545 --rpcaddr 127.0.0.1  --rpcapi "eth,net,web3,personal,miner,debug" --rpccorsdomain='*' --gasprice 0 --etherbase 0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2 --mine --miner.threads=1 --nodiscover
$ npx hardhat test --network geth
# specific test, e.g. TokenLocker test
$ npx hardhat test test/2_TokenLocker.test.js --network geth
```
