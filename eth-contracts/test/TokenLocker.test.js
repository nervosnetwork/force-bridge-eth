const { expect } = require("chai");
const { log, waitingForReceipt } = require("./utils");
const testJson = require("./data/testTokenLocker.json");

const recipientCellTypescript = testJson.recipientCellTypescript;
const decodeBurnTxTestCases = testJson.decodeBurnTxTestCases;
const lockETHTestCases = testJson.lockETHTestCases;
const lockTokenTestCases = testJson.lockTokenTestCases;

let tokenLocker, provider, user;

contract("TokenLocker", () => {
  before(async function () {
    // disable timeout
    this.timeout(0);

    let factory = await ethers.getContractFactory(
      "contracts/MockCKBSpv.sol:MockCKBSpv"
    );
    const mockSpv = await factory.deploy();
    await mockSpv.deployed();

    factory = await ethers.getContractFactory(
      "contracts/TokenLocker.sol:TokenLocker"
    );
    tokenLocker = await factory.deploy(
      mockSpv.address,
      123,
      recipientCellTypescript.codeHash,
      recipientCellTypescript.hashType
    );
    await tokenLocker.deployed();
    log("tokenLocker deployed to:", tokenLocker.address);

    user = tokenLocker.signer;
    provider = tokenLocker.provider;
  });

  describe("lockETH", async function () {
    // disable timeout
    this.timeout(0);
    it("Should lockETH verified", async () => {
      for (testcase of lockETHTestCases) {
        await testLockETH(testcase);
      }
    });
  });

  describe("lockToken", async function () {
    // disable timeout
    this.timeout(0);
    it("Should lock erc20 token verified", async () => {
      for (testcase of lockTokenTestCases) {
        await testLockToken(testcase);
      }
    });
  });

  describe("unlock token", async function () {
    // disable timeout
    this.timeout(0);
    it("should decode burn tx verified", async () => {
      for (t of decodeBurnTxTestCases) {
        const burnResult = await tokenLocker.decodeBurnResult(t.txData);
        expect(burnResult.token).to.equal(t.burnTokenAddress);
        expect(burnResult.recipient).to.equal(t.recipientAddress);
        expect(burnResult.amount.toNumber()).to.equal(t.burnAmount);
      }
    });
    // TODO test unlock
  });
});

async function testLockETH(testcase) {
  const contractBalance = await provider.getBalance(tokenLocker.address);

  // lockETH
  const amount = ethers.utils.parseEther(testcase.lockAmount);
  const res = await tokenLocker.lockETH(
    testcase.ckbAddress,
    testcase.replayResistOutpoint,
    { value: amount }
  );
  const receipt = await waitingForReceipt(provider, res);
  log(`gasUsed: ${receipt.gasUsed.toString()}`);

  const {
    tokenAddressTopic,
    lockerAddressTopic,
    ckbAddressTopic,
    lockedAmount,
    replayResistOutpoint,
  } = parseLockedEvent(receipt.logs[0]);

  expect(tokenAddressTopic).to.equal(
    "0x0000000000000000000000000000000000000000"
  );
  expect(lockerAddressTopic).to.equal(user.address);
  const hashCKBAddress = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes(testcase.ckbAddress)
  );
  expect(ckbAddressTopic).to.equal(hashCKBAddress);
  expect(lockedAmount).to.equal(amount);
  expect(replayResistOutpoint).to.equal(testcase.replayResistOutpoint);

  // locked token amount == delta balance of contract
  const delta =
    (await provider.getBalance(tokenLocker.address)) - contractBalance;
  const actualDelta = ethers.BigNumber.from(delta.toString());
  expect(actualDelta).to.equal(amount);
}

async function testLockToken(testcase) {
  const factory = await ethers.getContractFactory(
    "contracts/test/ERC20.sol:ERC20"
  );
  const erc20 = await factory.deploy();
  await erc20.deployed();
  log("erc20 deployed to:", erc20.address);

  const contractBalance = await erc20.callStatic.balanceOf(tokenLocker.address);
  log(`tokenLocker contract erc20 balance: ${contractBalance.toString()}`);
  const signerBalance = await erc20.callStatic.balanceOf(
    await user.getAddress()
  );
  log(`signer erc20 balance: ${signerBalance.toString()}`);

  // user should approve erc20 token to tokenLocker contract
  const approveAmount =
    "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
  let res = await erc20.approve(tokenLocker.address, approveAmount);
  let receipt = await waitingForReceipt(provider, res);
  log(`approve gasUsed: ${receipt.gasUsed.toString()}`);

  // lock erc20
  const amount = testcase.lockAmount;
  log(`lock erc20 token amount ${amount}`);
  res = await tokenLocker.lockToken(
    erc20.address,
    amount,
    testcase.ckbAddress,
    testcase.replayResistOutpoint
  );
  receipt = await waitingForReceipt(provider, res);
  log(`gasUsed: ${receipt.gasUsed.toString()}`);

  const {
    tokenAddressTopic,
    lockerAddressTopic,
    ckbAddressTopic,
    lockedAmount,
    replayResistOutpoint,
  } = parseLockedEvent(receipt.logs[2]);

  expect(tokenAddressTopic).to.equal(erc20.address);
  expect(lockerAddressTopic).to.equal(user.address);
  const hashCKBAddress = ethers.utils.keccak256(
    ethers.utils.toUtf8Bytes(testcase.ckbAddress)
  );
  expect(ckbAddressTopic).to.equal(hashCKBAddress);
  expect(lockedAmount).to.equal(amount);
  expect(replayResistOutpoint).to.equal(testcase.replayResistOutpoint);

  // locked token amount == delta balance of contract
  const contractBalanceAfter = await erc20.callStatic.balanceOf(
    tokenLocker.address
  );
  log(`contractBalanceAfter ${contractBalanceAfter}`);
  const delta = contractBalanceAfter - contractBalance;
  const actualDelta = ethers.BigNumber.from(delta.toString());
  expect(actualDelta).to.equal(amount);
}

function parseLockedEvent(eventLog) {
  const tokenAddressTopic = ethers.utils.defaultAbiCoder.decode(
    ["address"],
    eventLog.topics[1]
  )[0];
  const lockerAddressTopic = ethers.utils.defaultAbiCoder.decode(
    ["address"],
    eventLog.topics[2]
  )[0];
  const ckbAddressTopic = ethers.utils.defaultAbiCoder.decode(
    ["bytes32"],
    eventLog.topics[3]
  )[0];
  const eventData = ethers.utils.defaultAbiCoder.decode(
    ["uint256", "string"],
    eventLog.data
  );
  return {
    tokenAddressTopic: tokenAddressTopic,
    lockerAddressTopic: lockerAddressTopic,
    ckbAddressTopic: ckbAddressTopic,
    lockedAmount: eventData[0],
    replayResistOutpoint: eventData[1],
  };
}
