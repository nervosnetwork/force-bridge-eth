const { upgrades } = require('hardhat');
const { expect } = require('chai');
const { getTinyHeaders } = require('../benchmark/generateData');
const { keccak256, defaultAbiCoder, toUtf8Bytes } = ethers.utils;
const {
  log,
  sleep,
  waitingForReceipt,
  retryPromise,
} = require('../../test/utils');

const testJson = require('../../test/data/testTokenLocker.json');
const recipientCellTypescript = testJson.recipientCellTypescript;
const lightClientTypescriptHash = testJson.lightClientTypescriptHash;
const bridgeCellLockscriptCodeHash = testJson.bridgeCellLockscriptCodeHash;
const decodeBurnTxTestCases = testJson.decodeBurnTxTestCases;
const lockETHTestCases = testJson.lockETHTestCases;
const lockTokenTestCases = testJson.lockTokenTestCases;
let tokenLocker, provider, user;
const retryTimes = 20;
contract('TokenLocker openzeppelin upgradeable', () => {
  let adminAddress, contractAddress, initHeaderIndex, endHeaderIndex, factory;
  let wallets, validators;
  let multisigThreshold, chainId, DOMAIN_SEPARATOR, addHeadersTypeHash;

  before(async function () {
    // disable timeout
    this.timeout(0);

    let factory = await ethers.getContractFactory(
      'contracts/test/MockCKBSpv.sol:MockCKBSpv'
    );
    const mockSpv = await factory.deploy();
    await mockSpv.deployed();

    // deploy TokenLocker
    factory = await ethers.getContractFactory(
      'contracts/TokenLocker-openzeppelin.sol:TokenLocker'
    );
    tokenLocker = await retryPromise(
      upgrades.deployProxy(
        factory,
        [
          mockSpv.address,
          123,
          recipientCellTypescript.codeHash,
          recipientCellTypescript.hashType,
          lightClientTypescriptHash,
          bridgeCellLockscriptCodeHash,
        ],
        {
          initializer: 'initialize',
          unsafeAllowCustomTypes: true,
          unsafeAllowLinkedLibraries: true,
        }
      ),
      retryTimes
    );
    contractAddress = tokenLocker.address;
    provider = tokenLocker.provider;
    user = tokenLocker.signer;
  });

  describe('v1 test case', async function () {
    // disable timeout
    this.timeout(0);

    it('use v1 contract, lockETH correct case', async () => {
      for (const testcase of lockETHTestCases) {
        await testLockETH(testcase);
      }
    });
  });

  describe('upgrade v2 test case', async function () {
    // disable timeout
    this.timeout(0);

    it('upgrade to v2 tokenLocker', async () => {
      // deploy TokenLocker V2
      factory = await ethers.getContractFactory(
        'contracts/TokenLocker-openzeppelin-v2.sol:TokenLocker'
      );

      tokenLocker = await retryPromise(
        upgrades.upgradeProxy(contractAddress, factory, {
          unsafeAllowLinkedLibraries: true,
          unsafeAllowCustomTypes: true,
        }),
        retryTimes
      );

      await sleep(10);
    });

    it('use v2 contract, lockETH correct case', async () => {
      const key =
        '0x1234567890123456789012345678901234567890123456789012345678901234';
      const value =
        '0x1234567890123456789012345678901234567890123456789012345678901234';
      let res = await tokenLocker.testUpgrade1(key, value);
      await res.wait(1);
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
