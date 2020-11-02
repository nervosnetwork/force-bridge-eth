const { expect } = require('chai')
const { log, waitingForGasUsed } = require('./utils')
const vectors = require('./data/testSpv.json')

const {
  extractBlockNumber,
  extractBlockHash,
  expectedTransactionsRoot,
} = vectors


contract('HeaderVerifier', () => {
  let verifier, provider;

  before(async function () {
    // disable timeout
    this.timeout(0)
    const factory = await ethers.getContractFactory("contracts/CKBChain.sol:CKBChain")
    verifier = await factory.deploy()
    await verifier.deployed()
    provider = verifier.provider
  })

  describe('powMessage correct case', async function () {
    // disable timeout
    this.timeout(0)
    it('Should powMessage verified', async () => {
      // const headerData = '0x000000007ea9081ab0ccd0786e01000057040000000000000000005704cf0600258cb71cd81aeaca2e8141549322b85a5a83f1b3728d1bd860b4877453eba67b4313d9603ff24ef9197375a912562b88a94d7324c946195b9f8309763ef5c251937616899b34cc24a55012fe60cd46699df23d90ed35c5c49098dc55568c1d2b0000000000000000000000000000000000000000000000000000000000000000522e520916b1a12e161de18a0d872300a6e80f65451e000000efb310fe3dff0676030000312b000000000000216033d2';
      // const expectPowMsg = '0xcbecbaf6a2deee59b2eab3bbae5388128ce9f30183336526c9081419f163fc6076030000312b000000000000216033d2';
      // const res = await verifier._addHeader(headerData)
      // log('powMessage res = ', res)
    })
  })
})
