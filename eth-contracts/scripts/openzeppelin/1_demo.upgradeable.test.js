const { ethers, upgrades } = require('hardhat');
const { expect } = require('chai');
const { log, sleep, retryPromise } = require('../../test/utils');

contract('Box', () => {
  let contractAddress, provider;
  let factory;
  let box;
  let adminAddress;

  before(async function () {
    // disable timeout
    this.timeout(0);

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address;

    // deploy proxy contract
    factory = await ethers.getContractFactory(
      'contracts/test/openzeppelin/Box.sol:Box'
    );

    const retryTimes = 10;
    box = await retryPromise(
      upgrades.deployProxy(factory, [42], { initializer: 'store' }),
      retryTimes
    );

    await sleep(10);
    contractAddress = box.address;
    provider = box.provider;
  });

  describe('upgrade v1, v2, v3, v4 test case', async function () {
    // disable timeout
    this.timeout(0);

    it('use v1 contract', async function () {
      expect((await box.retrieve()).toString()).to.equal('42');
    });

    it('upgrade to v2, use v2 contract', async function () {
      factory = await ethers.getContractFactory(
        'contracts/test/openzeppelin/BoxV2.sol:BoxV2'
      );
      const contract = await upgrades.upgradeProxy(contractAddress, factory, {
        unsafeAllowLinkedLibraries: true,
        unsafeAllowCustomTypes: true,
      });

      log(`BoxV2 upgraded`);

      // waiting for block mined
      await sleep(10);

      let res = await contract.increment();
      await res.wait(1);
      res = await contract.retrieve();
      expect(res.toString()).to.equal('43');
    });

    it('upgrade to v3, use v3 contract', async function () {
      factory = await ethers.getContractFactory(
        'contracts/test/openzeppelin/BoxV3.sol:BoxV3'
      );
      const contract = await upgrades.upgradeProxy(contractAddress, factory, {
        unsafeAllowLinkedLibraries: true,
        unsafeAllowCustomTypes: true,
      });

      log(`BoxV3 upgraded`);

      // waiting for block mined
      await sleep(10);

      let res = await contract.increment();
      await res.wait(1);
      res = await contract.retrieve();
      expect(res.toString()).to.equal('2043');
    });

    it('upgrade to v4, use v4 contract( add struct )', async function () {
      factory = await ethers.getContractFactory(
        'contracts/test/openzeppelin/BoxV4.sol:BoxV4'
      );
      const contract = await upgrades.upgradeProxy(contractAddress, factory, {
        unsafeAllowLinkedLibraries: true,
        unsafeAllowCustomTypes: true,
      });

      log(`BoxV4 upgraded, with struct added`);

      // waiting for block mined
      await sleep(10);

      const tx = await contract.setHeader(
        124,
        '0x1234567890123456789012345678901234567890123456789012345678901235'
      );
      await tx.wait(1);
      const res = await contract.callStatic.currentHeader();
      expect(res[0]).to.equal(124);
      expect(res[1]).to.equal(
        '0x1234567890123456789012345678901234567890123456789012345678901235'
      );
    });
  });
});
