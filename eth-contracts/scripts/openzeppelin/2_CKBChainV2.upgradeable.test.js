const { ethers, upgrades } = require('hardhat');
const { expect } = require('chai');
const { getTinyHeaders } = require('../benchmark/generateData');
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils;
const {
  log,
  sleep,
  generateSignatures,
  generateWallets,
  retryPromise,
  getMsgHashForAddHeaders,
} = require('../../test/utils');
const retryTimes = 20;
contract('CKBChainV2 openzeppelin upgradeable', () => {
  let ckbChain,
    adminAddress,
    contractAddress,
    provider,
    initHeaderIndex,
    endHeaderIndex,
    factory;
  let wallets, validators;
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHeadersTypeHash;

  before(async function () {
    // disable timeout
    this.timeout(0);

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address;

    // get validators
    wallets = generateWallets(7);
    validators = wallets.map((wallet) => wallet.address);
    multisigThreshold = 5;
    chainId = await signer.getChainId();

    // deploy CKBChainV2
    const canonicalGcThreshold = 40;
    factory = await ethers.getContractFactory(
      'contracts/CKBChainV2-openzeppelin.sol:CKBChainV2'
    );

    ckbChain = await retryPromise(
      upgrades.deployProxy(
        factory,
        [canonicalGcThreshold, validators, multisigThreshold],
        {
          initializer: 'initialize',
          unsafeAllowCustomTypes: true,
          unsafeAllowLinkedLibraries: true,
        }
      ),
      retryTimes
    );
    contractAddress = ckbChain.address;
    provider = ckbChain.provider;
  });

  describe('upgrade v1, v2 test case', async function () {
    // disable timeout
    this.timeout(0);

    it('SIGNATURE_SIZE, name, ADD_HEADERS_TYPEHASH, DOMAIN_SEPARATOR', async () => {
      expect(await ckbChain.SIGNATURE_SIZE()).to.eq(65);

      const name = 'Force Bridge CKBChain';
      expect(await ckbChain.NAME_712()).to.eq(name);

      addHeadersTypeHash = keccak256(
        toUtf8Bytes('AddHeaders(bytes[] tinyHeaders)')
      );
      log(`addHeadersTypeHash`, addHeadersTypeHash);
      expect(await ckbChain.ADD_HEADERS_TYPEHASH()).to.eq(addHeadersTypeHash);

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
      );
      expect(await ckbChain.DOMAIN_SEPARATOR()).to.eq(DOMAIN_SEPARATOR);
    });

    it('use v1 contract, addHeaders correct case', async () => {
      let startIndex = 1;
      let actualTipNumber;
      const reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40];
      for (const size of reportSize) {
        const tinyHeaders = getTinyHeaders(startIndex, size);
        startIndex += size;

        // 1. calc msgHash
        const msgHash = getMsgHashForAddHeaders(
          DOMAIN_SEPARATOR,
          addHeadersTypeHash,
          tinyHeaders
        );

        // 2. generate signatures
        let signatures = generateSignatures(
          msgHash,
          wallets.slice(0, multisigThreshold)
        );

        // 3. addHeaders with gc
        const tx = await ckbChain.addHeaders(tinyHeaders, signatures);
        const receipt = await tx.wait(1);
        expect(receipt.status).to.eq(1);
        log(
          `add ${size} Headers gas: ${receipt.gasUsed}, gas cost per header: ${
            receipt.gasUsed / size
          }`
        );

        // check if addHeaders success
        actualTipNumber = await ckbChain.callStatic.latestBlockNumber();
        log(`current tipBlockNumber: ${actualTipNumber}\r\n`);
      }
    });

    it('upgrade to v2, use v2 contract', async function () {
      factory = await ethers.getContractFactory(
        'contracts/CKBChainV2-openzeppelin-upgrade-v2.sol:CKBChainV2'
      );

      const contract = await retryPromise(
        upgrades.upgradeProxy(contractAddress, factory, {
          unsafeAllowLinkedLibraries: true,
          unsafeAllowCustomTypes: true,
        }),
        retryTimes
      );

      log(`CKBChainV2 upgrade v2 upgraded`);

      // waiting for block mined
      await sleep(10);

      const key =
        '0x1234567890123456789012345678901234567890123456789012345678901234';
      const value =
        '0x1234567890123456789012345678901234567890123456789012345678901234';
      let res = await contract.testUpgrade1(key, value);
      await res.wait(1);
    });
  });
});
