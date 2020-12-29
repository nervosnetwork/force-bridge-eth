const chai = require('chai')
const vectors = require('../../data/testVectors.json')
const {getTinyHeaders} = require("../../../scripts/benchmark/generateData");
const { solidity } = require('ethereum-waffle')
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils
const {
  log,
  deployContract,
  generateSignatures,
  generateWallets,
  runErrorCase,
  deployUpgradableContractFirstTime,
  getMsgHashForAddHeaders,
} = require('../../utils')

chai.use(solidity)
const { expect } = chai
const {
  extractBlockNumber,
  calculateBlockHash,
  extractTransactionsRoot,
  extractEpoch,
  indexHeaderVec,
} = vectors

contract('CKBChainV2 upgradeable', () => {
  let ckbChain, adminAddress, provider, initHeaderIndex, endHeaderIndex, factory
  let wallets, validators
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHeadersTypeHash

  before(async function () {
    // disable timeout
    this.timeout(0)

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address;

    // get validators
    wallets = generateWallets(7)
    validators = wallets.map((wallet) => wallet.address)
    multisigThreshold = 5
    chainId = await signer.getChainId()

    // deploy CKBChainV2
    const canonicalGcThreshold = 40
    ckbChain = await deployUpgradableContractFirstTime(
        'contracts/CKBChainV2Storage.sol:CKBChainV2Storage',
        'contracts/CKBChainV2Logic.sol:CKBChainV2Logic',
        adminAddress,
        canonicalGcThreshold,
        validators,
        multisigThreshold,
    )
    log('CKBChainV2 deployed to:', ckbChain.address)
  })

  describe('addHeaders by multisig(5 of 7)', async function () {
    // disable timeout
    this.timeout(0)

    it('SIGNATURE_SIZE, name, ADD_HEADERS_TYPEHASH, DOMAIN_SEPARATOR', async () => {
      expect(await ckbChain.SIGNATURE_SIZE()).to.eq(65)

      const name = 'Force Bridge CKBChain'
      expect(await ckbChain.NAME_712()).to.eq(name)

      addHeadersTypeHash = keccak256(toUtf8Bytes('AddHeaders(bytes[] tinyHeaders)'))
      log(`addHeadersTypeHash`, addHeadersTypeHash)
      expect(await ckbChain.ADD_HEADERS_TYPEHASH()).to.eq(addHeadersTypeHash)

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
                ckbChain.address,
              ]
          )
      )
      expect(await ckbChain.DOMAIN_SEPARATOR()).to.eq(DOMAIN_SEPARATOR)
    })

    it('addHeaders correct case', async () => {
      let startIndex = 1
      let actualTipNumber
      const reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40]
      for (const size of reportSize) {
        const tinyHeaders = getTinyHeaders(startIndex, size)
        startIndex += size

        // 1. calc msgHash
        const msgHash = getMsgHashForAddHeaders(
            DOMAIN_SEPARATOR,
            addHeadersTypeHash,
            tinyHeaders
        )

        // 2. generate signatures
        let signatures = generateSignatures(
            msgHash,
            wallets.slice(0, multisigThreshold)
        )

        // 3. addHeaders with gc
        const tx = await ckbChain.addHeaders(tinyHeaders, signatures)
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

    it('addHeaders wrong cases', async () => {
      // get headers data
      let startIndex = 1
      const tinyHeaders = getTinyHeaders(startIndex, 1)

      // 1. calc msgHash
      const msgHash = getMsgHashForAddHeaders(
          DOMAIN_SEPARATOR,
          addHeadersTypeHash,
          tinyHeaders
      )

      // 2. generate signatures
      let signatures = generateSignatures(
          msgHash,
          wallets.slice(0, multisigThreshold)
      )

      // expect error of `invalid v of signature(r, s, v)`
      let wrongSignatures = signatures.slice(0, signatures.length - 2) + 'ff'
      await runErrorCase(
          ckbChain.addHeaders(tinyHeaders, wrongSignatures),
          'invalid v of signature(r, s, v)'
      )

      // expect error of `length of signatures must greater than threshold`
      wrongSignatures = signatures.slice(0, signatures.length - 65 * 2)
      await runErrorCase(
          ckbChain.addHeaders(tinyHeaders, wrongSignatures),
          'length of signatures must greater than threshold'
      )

      // expect error of `signatures not verified`
      wrongSignatures = signatures.slice(0, signatures.length - 65 * 2)
      wrongSignatures = wrongSignatures + wrongSignatures.slice(2)
      await runErrorCase(
          ckbChain.addHeaders(tinyHeaders, wrongSignatures),
          'signatures not verified'
      )
    })

  })
})
