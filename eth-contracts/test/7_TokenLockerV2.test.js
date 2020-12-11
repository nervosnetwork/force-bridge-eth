const chai = require('chai')
const { MaxUint256 } = ethers.constants
const {
  bigNumberify,
  hexlify,
  keccak256,
  defaultAbiCoder,
  toUtf8Bytes,
  solidityPack,
} = ethers.utils
const { solidity } = require('ethereum-waffle')

const { log, generateWallets, generateSignatures } = require('./utils')
const testJson = require('./data/testTokenLocker.json')

const recipientCellTypescript = testJson.recipientCellTypescript
const bridgeCellLockscriptCodeHash = testJson.bridgeCellLockscriptCodeHash

chai.use(solidity)
const { expect } = chai

const getMsgHashForSetNewCkbSpv = (
  DOMAIN_SEPARATOR,
  typeHash,
  newSpvAddress,
  nonce
) => {
  return keccak256(
    solidityPack(
      ['bytes1', 'bytes1', 'bytes32', 'bytes32'],
      [
        '0x19',
        '0x01',
        DOMAIN_SEPARATOR,
        keccak256(
          defaultAbiCoder.encode(
            ['bytes32', 'address', 'uint256'],
            [typeHash, newSpvAddress, nonce]
          )
        ),
      ]
    )
  )
}

contract('TokenLocker in TokenLockerV2', () => {
  let tokenLocker, provider, user
  let wallets, validators
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, newCkbSpvTypeHash

  before(async function () {
    // disable timeout
    this.timeout(0)

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

    // deploy TokenLockerV2
    factory = await ethers.getContractFactory(
      'contracts/TokenLockerV2.sol:TokenLocker'
    )
    tokenLocker = await factory.deploy(
      mockSpv.address,
      123,
      recipientCellTypescript.codeHash,
      recipientCellTypescript.hashType,
      bridgeCellLockscriptCodeHash,
      validators,
      multisigThreshold,
      chainId
    )
    await tokenLocker.deployed()
    log('tokenLocker deployed to:', tokenLocker.address)
  })

  describe('setNewCkbSpv by multisig(5 of 5)', async function () {
    // disable timeout
    this.timeout(0)

    it('SIGNATURE_SIZE, name, SET_NEW_CKB_SPV_TYPEHASH, DOMAIN_SEPARATOR', async () => {
      expect(await tokenLocker.SIGNATURE_SIZE()).to.eq(65)

      const name = 'Force Bridge TokenLocker'
      expect(await tokenLocker.name()).to.eq(name)

      newCkbSpvTypeHash = keccak256(
        toUtf8Bytes('SetNewCkbSpv(address newSpvAddress,uint256 nonce)')
      )
      expect(await tokenLocker.SET_NEW_CKB_SPV_TYPEHASH()).to.eq(
        newCkbSpvTypeHash
      )

      DOMAIN_SEPARATOR = keccak256(
        defaultAbiCoder.encode(
          ['bytes32', 'bytes32', 'bytes32', 'uint256', 'address'],
          [
            keccak256(
              toUtf8Bytes(
                'EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'
              )
            ),
            keccak256(toUtf8Bytes(name)),
            keccak256(toUtf8Bytes('1')),
            chainId,
            tokenLocker.address,
          ]
        )
      )
      expect(await tokenLocker.DOMAIN_SEPARATOR()).to.eq(DOMAIN_SEPARATOR)
    })

    it('setNewCkbSpv correct case', async () => {
      const newSpvAddress = '0x2222222222222222222222222222222222222222'
      const nonce = await tokenLocker.currentSetNewCkbSpvNonce()
      const msgHash = getMsgHashForSetNewCkbSpv(
        DOMAIN_SEPARATOR,
        newCkbSpvTypeHash,
        newSpvAddress,
        nonce
      )

      let signatures = generateSignatures(msgHash, wallets)
      let tx = await tokenLocker.setNewCkbSpv(newSpvAddress, nonce, signatures)
      let receipt = await tx.wait(1)
      expect(receipt.status).to.eq(1)
      expect(await tokenLocker.ckbSpv_()).to.eq(newSpvAddress)
      expect(await tokenLocker.currentSetNewCkbSpvNonce()).to.eq(nonce + 1)
    })

    it('setNewCkbSpv wrong cases', async () => {
      const newSpvAddress = '0x3333333333333333333333333333333333333333'
      const nonce = await tokenLocker.currentSetNewCkbSpvNonce()
      const msgHash = getMsgHashForSetNewCkbSpv(
        DOMAIN_SEPARATOR,
        newCkbSpvTypeHash,
        newSpvAddress,
        nonce
      )

      let signatures = generateSignatures(msgHash, wallets)
      // expect error of `invalid v of signature(r, s, v)`
      let wrongSignatures = signatures.slice(0, signatures.length - 2) + 'ff'
      try {
        await tokenLocker.setNewCkbSpv(newSpvAddress, nonce, wrongSignatures)
      } catch (e) {
        const error = e.error.toString()
        expect(error.indexOf('invalid v of signature(r, s, v)') > -1).to.eq(
          true
        )
      }

      // expect error of `length of signatures must greater than threshold`
      wrongSignatures = signatures.slice(0, signatures.length - 65 * 2)
      try {
        await tokenLocker.setNewCkbSpv(newSpvAddress, nonce, wrongSignatures)
      } catch (e) {
        const error = e.error.toString()
        expect(
          error.indexOf('length of signatures must greater than threshold') > -1
        ).to.eq(true)
      }

      // expect error of `signatures not verified`
      wrongSignatures = signatures.slice(0, signatures.length - 65 * 2)
      wrongSignatures = wrongSignatures + wrongSignatures.slice(2)
      try {
        await tokenLocker.setNewCkbSpv(newSpvAddress, nonce, wrongSignatures)
      } catch (e) {
        const error = e.error.toString()
        expect(error.indexOf('signatures not verified') > -1).to.eq(true)
      }

      // expect error of `invalid setNewCkbSpv nonce`
      const wrongNonce = nonce + 1
      try {
        await tokenLocker.setNewCkbSpv(
          newSpvAddress,
          wrongNonce,
          wrongSignatures
        )
      } catch (e) {
        const error = e.error.toString()
        expect(error.indexOf('invalid setNewCkbSpv nonce') > -1).to.eq(true)
      }
    })
  })
})
