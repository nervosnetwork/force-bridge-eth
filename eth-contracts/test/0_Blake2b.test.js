const _ = require('lodash');
const { ckbBlake2b, log } = require('./utils');

const generateRandomHex = () => {
  const chars = '0123456789abcdef';
  const len = chars.length;
  const hexLen = _.random(1, 10000) * 2;
  let res = '';
  for (let i = 0; i < hexLen; i++) {
    const index = _.random(0, len - 1);
    res += chars[index];
  }
  return '0x' + res;
};

const testTimes = 100;
contract('Blake2b', () => {
  let instance, factory;
  before(async function () {
    // disable timeout
    this.timeout(0);

    factory = await ethers.getContractFactory(
      'contracts/test/TestBlake2b.sol:TestBlake2b'
    );
    instance = await factory.deploy();
    await instance.deployTransaction.wait(1);
  });

  describe('#use ckbBlake2b to calc txHash', async () => {
    it(`${testTimes} ckbBlake2b random tests`, async () => {
      for (let i = 0; i < testTimes; i++) {
        const data = generateRandomHex();
        log(i, 'ckbBlake2b random test passed');
        const resActual = await instance.ckbBlake2bTx(data);
        const resExpected = ckbBlake2b(data);
        expect(resActual).to.equal(resExpected);
      }
    });
  });
});
