async function sleep(seconds) {
  // console.log(`waiting for block confirmations, about ${seconds}s`)
  await new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}

async function waitingForReceipt(provider, res) {
  if (!res) {
    return -1;
  }

  const txHash = res.hash;
  let txReceipt;
  while (!txReceipt) {
    txReceipt = await provider.getTransactionReceipt(txHash);
    if (txReceipt && txReceipt.blockHash) {
      break;
    }
    await sleep(1);
  }
  return txReceipt;
}

const deployContract = async (factoryPath, ...args) => {
  // const factory = await ethers.getContractFactory("contracts/token/NFI.sol:NFI");
  const factory = await ethers.getContractFactory(factoryPath);
  const contract = await factory.deploy(...args);

  // await contract.deployed();
  await contract.deployTransaction.wait(1);
  return contract;
};

const { log } = console;

module.exports = { sleep, log, waitingForReceipt, deployContract };
