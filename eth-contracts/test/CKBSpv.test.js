const { expect } = require('chai')
const { log, waitingForGasUsed } = require('./utils')

contract('CKBSpv', () => {
  let ckbChain, provider

  before(async function () {
    // disable timeout
    this.timeout(0);
    const factory = await ethers.getContractFactory('CKBChain')
    ckbChain = await factory.deploy()
    await ckbChain.deployed()
    provider = ckbChain.provider
  })

  describe('proveTxExist correct case', async () => {
    it('Should ', async () => {
      const blockHash = '0x1b1c39558cc783206d8dbf89ee6887abed8912125a48860b2bc70cbf586011bd'
      const transactionsRoot = '0xdc91bc580c482bcbd239d33be39b29fd20ecc69315b7f5129fa9460c0a4de21d'
      const blockNumber = 3028129
      const res = await ckbChain.mockForProveTxExist(blockNumber + 100, blockNumber, blockHash, transactionsRoot)
      await waitingForGasUsed(provider, res)
      expect(await ckbChain.callStatic.latestBlockNumber()).to.equal(blockNumber + 100)

      // proveTxExist
      const txProof = '0x0a0100001c0000001e000000260000004600000066000000860000001100a1342e00000000001b1c39558cc783206d8dbf89ee6887abed8912125a48860b2bc70cbf586011bd39e33c8ad2e7e4eb71610d2bcdfbb0cb0fde2f96418256914ad2f5be1d6e933145b4b9ade6d9429c6d7c8709bb56634b486f167e86cc5ea972356e2f25f6aab104000000385dfb0153a0e3aec760120c4e333a4a6bec91eeaca359ef714709588d23ca16e60708f897ac89126126bc246cc30ccb8e66c4158167560eb3329ed9f40f676e4e2b18dab3b90798b333312dea4cee0b8c148fc51c599687a881a36b4a4a42d1a8b2c004891cb36f6f7ec24cc30327391ff1b338ab2d623070389f31b67d720e'
      const proveRes = await ckbChain.proveTxExist(txProof, 100)
      const gasUsed = await waitingForGasUsed(provider, res)
      log(`proveTxExist gasUsed: ${gasUsed.toString()}`)
      expect(proveRes).to.equal(true)
    })
  })
})
