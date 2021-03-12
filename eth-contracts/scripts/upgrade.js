const { upgrades } = require('hardhat');
const { log } = require('../test/utils');

// @dev upgrade the new version contract without changing contract address
// 1. require the new version contract factory path to upgrade
// 2. require old contract address
const contractFactoryPath =
  'contracts/upgrades/TokenLocker-openzeppelin-v2.sol:TokenLocker';
const contractAddress = '0xc9cdDFc8cF48048442B74731551545a6aF81b0E4';

async function main() {
  const retryTimes = 10;
  for (let i = 0; i < retryTimes; i++) {
    try {
      await upgrade();
      log(`upgrade success!`);
      break;
    } catch (e) {
      log(e);
      log('retry deploy times: ', i);
    }
  }
}

async function upgrade() {
  const factory = await ethers.getContractFactory(contractFactoryPath);
  await upgrades.upgradeProxy(contractAddress, factory, {
    unsafeAllowLinkedLibraries: true,
    unsafeAllowCustomTypes: true,
  });
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
