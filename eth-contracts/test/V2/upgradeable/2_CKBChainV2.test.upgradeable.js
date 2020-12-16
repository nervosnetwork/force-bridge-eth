const chai = require('chai')
const vectors = require('../../data/testVectors.json')

const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils

const { solidity } = require('ethereum-waffle')
const {
  log,
  generateSignatures,
  generateWallets,
  runErrorCase,
  getMsgHashForAddHeaders,
  deployUpgradeabeContractFirstTime,
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

contract('CKBChainV2', () => {
  let ckbChain, provider, initHeaderIndex, endHeaderIndex
  let wallets, validators
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHeadersTypeHash
  let adminAddress;

  before(async function () {
    // disable timeout
    this.timeout(0)

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address;

    // get validators
    wallets = generateWallets(7)
    validators = wallets.map((wallet) => wallet.address)
    multisigThreshold = 5
    chainId = 1

    // deploy CKBChainV2
    // factory = await ethers.getContractFactory(
    //   'contracts/CKBChainV2.sol:CKBChainV2'
    // )
    // ckbChain = await factory.deploy(validators, multisigThreshold, chainId)
    // await ckbChain.deployed()

    ckbChain = await deployUpgradeabeContractFirstTime(
      'contracts/CKBChainV2Storage.sol:CKBChainV2Storage',
      'contracts/CKBChainV2Logic.sol:CKBChainV2Logic',
      adminAddress,
      validators,
      multisigThreshold,
      chainId
    )

    log('ckbChain deployed to:', ckbChain.address)
    provider = ckbChain.provider
    initHeaderIndex = extractBlockNumber.length - 3 // it will add 2 headers
  })

  describe('addHeaders by multisig(5 of 7)', async function () {
    // disable timeout
    this.timeout(0)

    it('SIGNATURE_SIZE, name, ADD_HEADERS_TYPEHASH, DOMAIN_SEPARATOR', async () => {
      expect(await ckbChain.SIGNATURE_SIZE()).to.eq(65)

      const name = 'Force Bridge CKBChain'
      expect(await ckbChain.NAME_712()).to.eq(name)

      addHeadersTypeHash = keccak256(toUtf8Bytes('AddHeaders(bytes data)'))
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

    it('Should initWithHeader success', async () => {
      const finalizedGcThreshold = 500
      const canonicalGcThreshold = 40000

      const initHeaderData = calculateBlockHash[initHeaderIndex].input
      const initBlockHash = calculateBlockHash[initHeaderIndex].output
      let res = await ckbChain.initWithHeader(
        initHeaderData,
        initBlockHash,
        finalizedGcThreshold,
        canonicalGcThreshold
      )
      let txReceipt = await res.wait(1)
      log(`initWithHeader gasUsed: ${txReceipt.gasUsed.toString()}`)

      // verify result
      let expectTipNumber = extractBlockNumber[initHeaderIndex].output
      let actualTipNumber = await ckbChain.callStatic.getLatestBlockNumber()
      expect(actualTipNumber).to.equal(expectTipNumber)

      let expectCanonicalHeaderHash = initBlockHash
      let actualCanonicalHeaderHash = await ckbChain.callStatic.getCanonicalHeaderHash(
        expectTipNumber
      )
      expect(actualCanonicalHeaderHash).to.equal(expectCanonicalHeaderHash)

      let expectLatestEpoch = extractEpoch[initHeaderIndex].output
      let actualLatestEpoch = await ckbChain.callStatic.getLatestEpoch()
      expect(actualLatestEpoch).to.equal(expectLatestEpoch)

      let expectTransactionsRoot =
        extractTransactionsRoot[initHeaderIndex].output
      let actualTransactionsRoot = await ckbChain.callStatic.getCanonicalTransactionsRoot(
        initBlockHash
      )
      expect(actualTransactionsRoot).to.equal(expectTransactionsRoot)
    })

    it('addHeaders correct case', async () => {
      const startIndex = initHeaderIndex + 1
      // get headers data
      const headersInput = indexHeaderVec[startIndex].input
      const headers = indexHeaderVec[startIndex].output

      const msgHash = getMsgHashForAddHeaders(
        DOMAIN_SEPARATOR,
        addHeadersTypeHash,
        headersInput
      )
      let signatures = generateSignatures(
        msgHash,
        wallets.slice(0, multisigThreshold)
      )

      let tx = await ckbChain.addHeaders(headersInput, signatures)
      let receipt = await tx.wait(1)
      expect(receipt.status).to.eq(1)

      // verify result
      endHeaderIndex = startIndex + headers.length - 1
      let expectTipNumber = extractBlockNumber[endHeaderIndex].output
      let actualTipNumber = await ckbChain.callStatic.getLatestBlockNumber()
      expect(actualTipNumber).to.equal(expectTipNumber)

      for (let i = 0; i < headers.length; i++) {
        const headerIndex = startIndex + i
        let expectBlockHash = calculateBlockHash[headerIndex].output
        let blockNumber = extractBlockNumber[headerIndex].output
        let actualBlockHash = await ckbChain.callStatic.getCanonicalHeaderHash(
          blockNumber
        )
        expect(actualBlockHash).to.equal(expectBlockHash)
      }
    })

    it('setNewCkbSpv wrong cases', async () => {
      const startIndex = initHeaderIndex + 1
      // get headers data
      const headersInput = indexHeaderVec[startIndex].input
      const msgHash = getMsgHashForAddHeaders(
        DOMAIN_SEPARATOR,
        addHeadersTypeHash,
        headersInput
      )

      let signatures = generateSignatures(msgHash, wallets)
      // expect error of `invalid v of signature(r, s, v)`
      let wrongSignatures = signatures.slice(0, signatures.length - 2) + 'ff'
      await runErrorCase(
        ckbChain.addHeaders(headersInput, wrongSignatures),
        'invalid v of signature(r, s, v)'
      )

      // expect error of `length of signatures must greater than threshold`
      wrongSignatures = signatures.slice(0, signatures.length - 65 * 2)
      await runErrorCase(
        ckbChain.addHeaders(headersInput, wrongSignatures),
        'length of signatures must greater than threshold'
      )

      // expect error of `signatures not verified`
      wrongSignatures = signatures.slice(0, signatures.length - 65 * 2)
      wrongSignatures = wrongSignatures + wrongSignatures.slice(2)
      await runErrorCase(
        ckbChain.addHeaders(headersInput, wrongSignatures),
        'signatures not verified'
      )
    })
  })
})
