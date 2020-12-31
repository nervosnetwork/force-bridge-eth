const { expect } = require('chai');
const vectors = require('../data/testVectors.json');

const { calculateEaglesong } = vectors;

contract('Eaglesong', () => {
  let eaglesong, testEaglesong;
  let provider;

  before(async function () {
    // disable timeout
    this.timeout(0);

    // deploy hamstersong
    let factory = await ethers.getContractFactory(
      'contracts/Eaglesong.sol:Eaglesong'
    );
    eaglesong = await factory.deploy();
    await eaglesong.deployTransaction.wait(1);
    const hamAddr = eaglesong.address;

    // deploy TestEaglesong
    factory = await ethers.getContractFactory(
      'contracts/test/TestEaglesong.sol:TestEaglesong'
    );
    testEaglesong = await factory.deploy(hamAddr);
    await testEaglesong.deployTransaction.wait(1);
    provider = testEaglesong.provider;
  });

  describe('Eaglesong correct case', async function () {
    // disable timeout
    this.timeout(0);
    it('Should Eaglesong verified', async () => {
      for (let i = 0; i < calculateEaglesong.length; i++) {
        // calc Eaglesong
        const data = calculateEaglesong[i].input;
        let res = await testEaglesong.callStatic.ckbEaglesong(data);
        expect(res).to.equal(calculateEaglesong[i].output);

        // calc gas
        await testEaglesong.ckbEaglesong(data);
      }
    });
  });
});
