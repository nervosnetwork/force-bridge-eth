const chai = require('chai')
const { keccak256, defaultAbiCoder, toUtf8Bytes, solidityPack } = ethers.utils
const { solidity } = require('ethereum-waffle')

const {
  log,
  generateWallets,
  generateSignatures,
  runErrorCase,
  getMsgHashForSetNewCkbSpv,
  deployUpgradableContractFirstTime
} = require('../../utils')
const testJson = require('../../data/testTokenLocker.json')

const recipientCellTypescript = testJson.recipientCellTypescript
const lightClientTypescriptHash = testJson.lightClientTypescriptHash
const bridgeCellLockscriptCodeHash = testJson.bridgeCellLockscriptCodeHash

chai.use(solidity)
const { expect } = chai

contract('TokenLocker in TokenLockerV2', () => {
  let tokenLocker, provider, user
  let wallets, validators
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, newCkbSpvTypeHash
  let adminAddress;

  before(async function () {
    // disable timeout
    this.timeout(0)

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address;

    let factory = await ethers.getContractFactory(
      'contracts/test/MockCKBSpv.sol:MockCKBSpv'
    )
    const mockSpv = await factory.deploy()
    await mockSpv.deployed()
    user = mockSpv.signer
    provider = mockSpv.provider

    // get validators
    wallets = generateWallets(5)
    validators = wallets.map((wallet) => wallet.address)
    multisigThreshold = 5
    chainId = await user.getChainId()

    tokenLocker = await deployUpgradableContractFirstTime(
      'contracts/TokenLockerStorage.sol:TokenLockerV2Storage',
      'contracts/TokenLockerLogic.sol:TokenLockerV2Logic',
      adminAddress,
      mockSpv.address,
      123,
      recipientCellTypescript.codeHash,
      recipientCellTypescript.hashType,
      lightClientTypescriptHash,
      bridgeCellLockscriptCodeHash,
      validators,
      multisigThreshold,
      chainId
    )

    log('tokenLocker deployed to:', tokenLocker.address)
  })

  describe('setNewCkbSpv by multisig(5 of 5)', async function () {
    // disable timeout
    this.timeout(0)

  })
})
