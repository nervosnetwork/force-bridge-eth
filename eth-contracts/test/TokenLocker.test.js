const { expect } = require('chai')
const { log, waitingForGasUsed } = require('./utils')

contract('TokenLocker', () => {
  let tokenLocker; let
    provider

  before(async function () {
    // disable timeout
    this.timeout(0)
    let factory = await ethers.getContractFactory("contracts/MockCKBSpv.sol:MockCKBSpv")
    const mockSpv = await factory.deploy()
    await mockSpv.deployed()

    factory = await ethers.getContractFactory("contracts/TokenLocker.sol:TokenLocker")
    tokenLocker = await factory.deploy(mockSpv.address, 123)
    await tokenLocker.deployed()
    provider = tokenLocker.provider
  })

  describe('lockETH', async () => {
    it('Should ', async () => {
      // let defaultProvider = ethers.getDefaultProvider()
      const defaultProvider = tokenLocker.signer.provider
      const contractBalance = await defaultProvider.getBalance(tokenLocker.address)

      // lockETH
      const amount = ethers.utils.parseEther('1.2')
      const res = await tokenLocker.lockETH(0, { value: amount })
      const gasUsed = await waitingForGasUsed(provider, res)
      log(`gasUsed: ${gasUsed.toString()}`)

      // asset expected amount == balance of contract delta
      const delta = await defaultProvider.getBalance(tokenLocker.address) - contractBalance
      const actualDelta = ethers.BigNumber.from(delta.toString())
      expect(actualDelta).to.equal(amount)
    })
  })

  describe('unlockETH', async function () {
    // disable timeout
    this.timeout(0)
    it('Should ', async () => {
      // let defaultProvider = ethers.getDefaultProvider()
      const defaultProvider = tokenLocker.signer.provider
      const contractBalance = await defaultProvider.getBalance(tokenLocker.address)

      // unlockETH
      const res = await tokenLocker.unlockToken([0], [0])
      const gasUsed = await waitingForGasUsed(provider, res)
      log(`gasUsed: ${gasUsed.toString()}`)

      // asset expected amount == balance of contract delta
      const delta = await defaultProvider.getBalance(tokenLocker.address) - contractBalance
      const actualDelta = ethers.BigNumber.from(delta.toString())
      const expected = ethers.BigNumber.from('-111100000000000000')
      expect(actualDelta).to.equal(expected)
    })
  })
})
