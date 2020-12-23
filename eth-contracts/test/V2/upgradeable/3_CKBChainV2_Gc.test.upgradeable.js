const chai = require('chai')
const _ = require('lodash');
const {getHeaderAndHash, getHeadersVecAndHashes} = require("../../../scripts/benchmark/generateData");
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils
const { solidity } = require('ethereum-waffle')
const {
  log,
  generateSignatures,
  generateWallets,
  getMsgHashForAddHeaders,
  deployUpgradeabeContractFirstTime,
} = require('../../utils')

chai.use(solidity)
const { expect } = chai

contract('CKBChainV2 With Gc', () => {
  let ckbChain
  let wallets, validators
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHeadersTypeHash
  let adminAddress;

  before(async function () {
    // disable timeout
    this.timeout(0)

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address;

    // get validators
    wallets = generateWallets(10)
    validators = wallets.map((wallet) => wallet.address)
    multisigThreshold = 4
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
  })

  describe('addHeaders enhanced by multisig(4 of 10)', async function () {
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
      const finalizedGcThreshold = 50
      const canonicalGcThreshold = 100

      const [initHeaderData, initBlockHash] = getHeaderAndHash(0)
      let res = await ckbChain.initWithHeader(
        initHeaderData,
        initBlockHash,
        finalizedGcThreshold,
        canonicalGcThreshold
      )
      let txReceipt = await res.wait(1)
      log(`initWithHeader gasUsed: ${txReceipt.gasUsed.toString()}`)
    })

    it('addHeaders with Gc by random signatures', async () => {
      let startIndex = 1
      let actualTipNumber
      const allHashes = []
      const reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40]
      for (const size of reportSize) {
        const [headers, hashes] = getHeadersVecAndHashes(startIndex, size)
        allHashes.push(...hashes)
        startIndex += size

        // 1. generate msgHash
        const msgHash = getMsgHashForAddHeaders(
            DOMAIN_SEPARATOR,
            addHeadersTypeHash,
            headers
        )

        // 2. generate random signatures for msgHash
        const shuffledWallets = _.shuffle(wallets)
        const randomSize = _.random(multisigThreshold, shuffledWallets.length)
        const signatures = generateSignatures(
            msgHash,
            shuffledWallets.slice(0, randomSize)
        )

        // 3. addHeaders with gc
        const tx = await ckbChain.addHeaders(headers, signatures)
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
