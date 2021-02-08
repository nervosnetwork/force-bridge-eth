const { ethers } = require('hardhat');
const { expect } = require('chai');
const { getTinyHeaders } = require('../scripts/benchmark/generateData');
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils;
const {
  log,
  generateSignatures,
  generateWallets,
  getMsgHashForAddHistoryTxRoot,
} = require('./utils');

const { addHistoryTxRootTestCases } = require('./data/testHistoryTxRoot.json');
const retryTimes = 20;
contract('CKBChain', () => {
  let ckbChain, adminAddress, contractAddress, provider, factory;
  let wallets, validators;
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHistoryTxRootTypeHash;
  let initBlockNumber, latestBlockNumber, historyTxRoot, txRootProofDataVec;

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

    // deploy CKBChain
    const canonicalGcThreshold = 40;
    factory = await ethers.getContractFactory(
      'contracts/CKBChain.sol:CKBChain'
    );

    ckbChain = await factory.deploy();
    await ckbChain.deployTransaction.wait(1);
    const res = await ckbChain.initialize(validators, multisigThreshold);
    await res.wait(1);

    contractAddress = ckbChain.address;
    provider = ckbChain.provider;
  });

  describe('correct case', async function () {
    // disable timeout
    this.timeout(0);

    it('SIGNATURE_SIZE, name, AddHistoryTxRootTypeHash, DOMAIN_SEPARATOR', async () => {
      expect(await ckbChain.SIGNATURE_SIZE()).to.eq(65);

      const name = 'Force Bridge CKBChain';
      expect(await ckbChain.NAME_712()).to.eq(name);

      addHistoryTxRootTypeHash = keccak256(
        toUtf8Bytes(
          'AddHistoryTxRoot(uint64 startBlockNumber, uint64 endBlockNumber, bytes32 historyTxRoot)'
        )
      );
      log(`addHeadersTypeHash`, addHistoryTxRootTypeHash);
      expect(await ckbChain.ADD_HISTORY_TX_ROOT_TYPEHASH()).to.eq(
        addHistoryTxRootTypeHash
      );

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

    it('use v1 contract, addHistoryTxRoot correct case', async () => {
      let actualTipNumber;
      for (const testCase of addHistoryTxRootTestCases) {
        initBlockNumber = testCase.initBlockNumber;
        latestBlockNumber = testCase.latestBlockNumber;
        historyTxRoot = testCase.historyTxRoot;
        txRootProofDataVec = testCase.txRootProofDataVec;
        // 1. calc msgHash
        const msgHash = getMsgHashForAddHistoryTxRoot(
          DOMAIN_SEPARATOR,
          addHistoryTxRootTypeHash,
          initBlockNumber,
          latestBlockNumber,
          historyTxRoot
        );

        // 2. generate signatures
        let signatures = generateSignatures(
          msgHash,
          wallets.slice(0, multisigThreshold)
        );

        // 3. addHeaders with gc
        const tx = await ckbChain.addHistoryTxRoot(
          initBlockNumber,
          latestBlockNumber,
          historyTxRoot,
          signatures
        );
        const receipt = await tx.wait(1);
        expect(receipt.status).to.eq(1);
        log(`gas cost: ${receipt.gasUsed}`);

        // check if addHeaders success
        actualTipNumber = await ckbChain.callStatic.latestBlockNumber();
        log(`current tipBlockNumber: ${actualTipNumber}\r\n`);

        // // 4. proveTxRootExist
        // for (const proofData of txRootProofDataVec) {
        //   const res = await ckbChain.callStatic.proveTxRootExist(proofData);
        //   expect(res).to.equal(true);
        // }
      }
    });
  });
});
