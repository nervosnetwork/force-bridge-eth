const {
  sleep,
  log,
  waitingForReceipt,
  deployContract,
} = require("../../test/utils");

const hre = require("hardhat");

const benchmark = async (factoryPath) => {
  let res, receipt;

  // 1. deploy
  const contract = await deployContract(factoryPath);
  const provider = contract.provider;
  const signer = contract.signer;

  // - benchmark no gc
  res = await contract.setStorage();
  receipt = await res.wait(1);
  log(`100 kv insert, gasUsed: ${receipt.gasUsed}`);

  // - benchmark gc
  res = await contract.testGcGas();
  receipt = await res.wait(1);
  log(`100 kv insert with deleting 100 kv,  gasUsed: ${receipt.gasUsed}`);

  // - benchmark for loo
  // init
  res = await contract.setStorage();
  await res.wait(1);

  res = await contract.testQueueGas();
  receipt = await res.wait(1);
  log(`100 kv insert using loop queue,  gasUsed: ${receipt.gasUsed}`);
};

const main = async () => {
  await benchmark("contracts/test/TestGcGas.sol:TestGcGas");
};

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
