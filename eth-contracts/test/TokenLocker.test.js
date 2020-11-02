const { expect } = require('chai')
const { log, waitingForGasUsed } = require('./utils')

contract('TokenLocker', () => {
  let tokenLocker, provider, user

  before(async function () {
    // disable timeout
    this.timeout(0)
    let factory = await ethers.getContractFactory("contracts/MockCKBSpv.sol:MockCKBSpv")
    const mockSpv = await factory.deploy()
    await mockSpv.deployed()

    factory = await ethers.getContractFactory("contracts/TokenLocker.sol:TokenLocker")
    tokenLocker = await factory.deploy(mockSpv.address, 123)
    await tokenLocker.deployed()

    user = tokenLocker.signer
    provider = tokenLocker.provider
  })

  describe('lockETH', async function () {
    // disable timeout
    this.timeout(0)
    it('Should lockETH verified', async () => {
      // let defaultProvider = ethers.getDefaultProvider()
      const contractBalance = await provider.getBalance(tokenLocker.address)

      // lockETH
      const amount = ethers.utils.parseEther('1.2')
      const res = await tokenLocker.lockETH(0, { value: amount })
      const gasUsed = await waitingForGasUsed(provider, res)
      log(`gasUsed: ${gasUsed.toString()}`)

      // asset expected amount == balance of contract delta
      const delta = await provider.getBalance(tokenLocker.address) - contractBalance
      const actualDelta = ethers.BigNumber.from(delta.toString())
      expect(actualDelta).to.equal(amount)
    })
  })

  describe('lockToken', async function () {
    // disable timeout
    this.timeout(0)
    it('Should lock erc20 token verified', async () => {
      // deploy erc20
      const factory = await ethers.getContractFactory("contracts/test/ERC20.sol:ERC20")
      const erc20 = await factory.deploy()
      await erc20.deployed()
      log("erc20 deployed to:", erc20.address);

      // get balance
      const contractBalance = await erc20.callStatic.balanceOf(tokenLocker.address)
      log(`tokenLocker contract erc20 balance: ${contractBalance.toString()}`)
      const signerBalance = await erc20.callStatic.balanceOf(await user.getAddress())
      log(`signer erc20 balance: ${signerBalance.toString()}`)

      // user should approve erc20 token to tokenLocker contract
      const approveAmount = '0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF'
      let res = await erc20.approve(tokenLocker.address, approveAmount)
      let gasUsed = await waitingForGasUsed(provider, res)
      log(`approve gasUsed: ${gasUsed.toString()}`)

      // lock erc20
      const amount = 789
      log(`lock erc20 token amount ${amount}`)
      res = await tokenLocker.lockToken(erc20.address, amount, "ckb_address")
      gasUsed = await waitingForGasUsed(provider, res)
      log(`gasUsed: ${gasUsed.toString()}`)

      // asset expected amount == balance of contract delta
      const contractBalanceAfter = await erc20.callStatic.balanceOf(tokenLocker.address)
      log(`contractBalanceAfter ${contractBalanceAfter}`)
      const delta = contractBalanceAfter - contractBalance
      const actualDelta = ethers.BigNumber.from(delta.toString())
      expect(actualDelta).to.equal(amount)
    })
  })

  describe('unlockETH', async function () {
    // disable timeout
    this.timeout(0)
    it('Should unlockETH verified', async () => {
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
