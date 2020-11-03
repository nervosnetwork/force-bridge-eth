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
