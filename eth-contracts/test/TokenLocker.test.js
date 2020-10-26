const { expect } = require('chai');

const WAITING_SECONDS = 10
async function sleep(seconds) {
  console.log(`waiting for block confirmations, about ${seconds}s`)
  await new Promise(resolve => setTimeout(resolve, seconds * 1000))
}

contract('TokenLocker', () => {
  let tokenLocker;

  before(async () => {
    let factory = await ethers.getContractFactory('MockCKBSpv');
    const mockSpv = await factory.deploy();
    await mockSpv.deployed();
    await sleep(WAITING_SECONDS);

    factory = await ethers.getContractFactory('TokenLocker');
    tokenLocker = await factory.deploy(mockSpv.address, 123);
    await tokenLocker.deployed();
    await sleep(WAITING_SECONDS);
  });

  describe('lockETH', async () => {
    it('Should ', async () => {
      // let defaultProvider = ethers.getDefaultProvider();
      const defaultProvider = tokenLocker.signer.provider;
      const contractBalance = await defaultProvider.getBalance(tokenLocker.address);

      // lockETH
      const amount = ethers.utils.parseEther('1.2');
      await tokenLocker.lockETH(0, { value: amount });

      await sleep(WAITING_SECONDS);

      // asset expected amount == balance of contract delta
      const delta = await defaultProvider.getBalance(tokenLocker.address) - contractBalance;
      const actualDelta = ethers.BigNumber.from(delta.toString());
      expect(actualDelta).to.equal(amount);
    });
  });

  describe('unlockETH', async () => {
    it('Should ', async () => {
      // let defaultProvider = ethers.getDefaultProvider();
      const defaultProvider = tokenLocker.signer.provider;
      const contractBalance = await defaultProvider.getBalance(tokenLocker.address);

      // unlockETH
      await tokenLocker.unlockToken([0], [0]);
      await sleep(WAITING_SECONDS)

      // asset expected amount == balance of contract delta
      const delta = await defaultProvider.getBalance(tokenLocker.address) - contractBalance;
      const actualDelta = ethers.BigNumber.from(delta.toString());
      const expected = ethers.BigNumber.from('-111100000000000000');
      expect(actualDelta).to.equal(expected);
    });
  });
});
