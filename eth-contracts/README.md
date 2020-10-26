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
$ npx buidler test
```
## Test on Testnet
```bash
# create .env file for testnet-ropsten, make sure you have enough ethers in your Ropsten account for delopy the contract
$ echo ROPSTEN_DEPLOYER_PRIVATE_KEY=\"0x0000000000000000000000000000000000000000000000000000000000000000\"\\r\\n\
  ROPSTEN_API=\"https://ropsten.infura.io/v3/3ed3eadf912c4b31b800aafeedbf79eb\" >> .env
$ npx buidler run scripts/testnet-ropsten/test-proveTxExist.js --network ropsten
```
