const { expect } = require('chai');
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils;
const {
  log,
  waitingForReceipt,
  generateSignatures,
  generateWallets,
  getMsgHashForAddHistoryTxRoot,
} = require('./utils');
const { addHistoryTxRootTestCases } = require('./data/testHistoryTxRoot.json');
const viewHistoryTxProof = require('./data/testHistoryTxProof.json');
const testJson = require('./data/testTokenLocker.json');
const recipientCellTypescript = testJson.recipientCellTypescript;
const lightClientTypescriptHash = testJson.lightClientTypescriptHash;
const bridgeCellLockscriptCodeHash = testJson.bridgeCellLockscriptCodeHash;
const decodeBurnTxTestCases = testJson.decodeBurnTxTestCases;
const lockETHTestCases = testJson.lockETHTestCases;
const lockTokenTestCases = testJson.lockTokenTestCases;
const unlockTokenTestCase = require('./data/testUnlockTokenParam.json');
let tokenLocker, provider, user;
const retryTimes = 20;
contract('TokenLocker', () => {
  let ckbChain,
    adminAddress,
    contractAddress,
    initHeaderIndex,
    endHeaderIndex,
    factory;
  let wallets, validators;
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHistoryTxRootTypeHash;
  let initBlockNumber,
    latestBlockNumber,
    historyTxRoot,
    txRootProofDataVec,
    input;

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
    factory = await ethers.getContractFactory(
      'contracts/CKBChain.sol:CKBChain'
    );

    ckbChain = await factory.deploy();
    await ckbChain.deployTransaction.wait(1);
    let res = await ckbChain.initialize(validators, multisigThreshold);
    await res.wait(1);

    // deploy TokenLocker
    factory = await ethers.getContractFactory(
      'contracts/test/TestTokenLocker.sol:TestTokenLocker'
    );
    tokenLocker = await factory.deploy();
    await tokenLocker.deployTransaction.wait(1);
    res = await tokenLocker.initialize(
      ckbChain.address,
      0,
      recipientCellTypescript.codeHash,
      recipientCellTypescript.hashType,
      lightClientTypescriptHash,
      bridgeCellLockscriptCodeHash
    );
    await res.wait(1);

    contractAddress = tokenLocker.address;
    provider = tokenLocker.provider;
    user = tokenLocker.signer;
  });

  describe('ckbChain addHistoryTxRoot', async function () {
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
        input = testCase.input;
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
      }
    });
  });

  describe('tokenLocker test case', async function () {
    // disable timeout
    this.timeout(0);

    it('test _proveTxRootExist', async () => {
      const res = await tokenLocker.callStatic.testProveTxRootExist(
        input,
        historyTxRoot
      );
      expect(res).to.eq(true);
    });

    it('test _proveTxExist', async () => {
      for (let i = 0; i < viewHistoryTxProof.calcTxHash.length; i++) {
        const testCase = viewHistoryTxProof.calcTxHash[i];
        const txRoot = viewHistoryTxProof.expectTransactionsRoot[i].output;
        const res = await tokenLocker.callStatic.testProveTxExist(
          testCase.input,
          testCase.output,
          txRoot
        );
        expect(res).to.eq(true);
      }
    });

    it('lockETH correct case', async () => {
      for (const testcase of lockETHTestCases) {
        await testLockETH(testcase);
      }
    });
  });
});

async function testLockETH(testcase) {
  const contractBalance = await provider.getBalance(tokenLocker.address);

  // lockETH
  const amount = ethers.utils.parseEther(testcase.lockAmount);
  const fee = ethers.utils.parseEther(testcase.bridgeFee);
  const res = await tokenLocker.lockETH(
    fee,
    testcase.recipientLockscript,
    testcase.replayResistOutpoint,
    testcase.sudtExtraData,
    { value: amount }
  );
  const receipt = await waitingForReceipt(provider, res);
  log(`gasUsed: ${receipt.gasUsed.toString()}`);

  const {
    tokenAddressTopic,
    lockerAddressTopic,
    lockedAmount,
    bridgeFee,
    recipientLockscript,
    replayResistOutpoint,
    sudtExtraData,
  } = parseLockedEvent(receipt.logs[0]);

  expect(tokenAddressTopic).to.equal(
    '0x0000000000000000000000000000000000000000'
  );
  expect(lockerAddressTopic).to.equal(user.address);
  expect(lockedAmount).to.equal(amount);
  expect(bridgeFee).to.equal(fee);
  expect(recipientLockscript).to.equal(testcase.recipientLockscript);
  expect(replayResistOutpoint).to.equal(testcase.replayResistOutpoint);
  expect(sudtExtraData).to.equal(testcase.sudtExtraData);

  // locked token amount == delta balance of contract
  const delta =
    (await provider.getBalance(tokenLocker.address)) - contractBalance;
  const actualDelta = ethers.BigNumber.from(delta.toString());
  expect(actualDelta).to.equal(amount);
}

function parseLockedEvent(eventLog) {
  const tokenAddressTopic = ethers.utils.defaultAbiCoder.decode(
    ['address'],
    eventLog.topics[1]
  )[0];
  const lockerAddressTopic = ethers.utils.defaultAbiCoder.decode(
    ['address'],
    eventLog.topics[2]
  )[0];
  const eventData = ethers.utils.defaultAbiCoder.decode(
    ['uint256', 'uint256', 'bytes', 'bytes', 'bytes'],
    eventLog.data
  );
  return {
    tokenAddressTopic: tokenAddressTopic,
    lockerAddressTopic: lockerAddressTopic,
    lockedAmount: eventData[0],
    bridgeFee: eventData[1],
    recipientLockscript: eventData[2],
    replayResistOutpoint: eventData[3],
    sudtExtraData: eventData[4],
  };
}
