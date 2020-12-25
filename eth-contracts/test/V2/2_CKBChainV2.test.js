const chai = require('chai')
const vectors = require('../data/testVectors.json')
const {getTinyHeaders} = require("../../scripts/benchmark/generateData");
const { solidity } = require('ethereum-waffle')
const {
  log,
  deployContract,
  generateSignatures,
  generateWallets,
} = require('../utils')

chai.use(solidity)
const { expect } = chai
const {
  extractBlockNumber,
  calculateBlockHash,
  extractTransactionsRoot,
  extractEpoch,
  indexHeaderVec,
} = vectors

contract('CKBChainV2', () => {
  let ckbChain, provider, initHeaderIndex, endHeaderIndex, factory
  let wallets, validators
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHeadersTypeHash

  before(async function () {
    // disable timeout
    this.timeout(0)

    // deploy CKBChainV2
    const canonicalGcThreshold = 40
    ckbChain = await deployContract('contracts/CKBChainV2.sol:CKBChainV2', canonicalGcThreshold)
    log('CKBChainV2 deployed to:', ckbChain.address)
  })

  describe('addHeaders by multisig(5 of 7)', async function () {
    // disable timeout
    this.timeout(0)

    it('addHeaders correct case', async () => {
      let startIndex = 1
      let actualTipNumber
      const reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40]
      for (const size of reportSize) {
        const tinyHeaders = getTinyHeaders(startIndex, size)
        startIndex += size

        // 3. addHeaders with gc
        const tx = await ckbChain.addHeaders(tinyHeaders)
        const receipt = await tx.wait(1)
        expect(receipt.status).to.eq(1)
        log(
            `add ${size} Headers gas: ${receipt.gasUsed}, gas cost per header: ${
                receipt.gasUsed / size
            }`
        )

        // check if addHeaders success
        actualTipNumber = await ckbChain.callStatic.getLatestBlockNumber()
        log(`current tipBlockNumber: ${actualTipNumber}\r\n`)
      }
    })
  })
})
