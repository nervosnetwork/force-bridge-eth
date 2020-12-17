const { ecsign, toRpcSig } = require('ethereumjs-util');
const { keccak256, defaultAbiCoder, solidityPack } = ethers.utils;

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

const deployAll = async (contractPaths) => {
  const contracts = [];
  const promises = [];
  for (const path of contractPaths) {
    const factory = await ethers.getContractFactory(path);
    const contract = await factory.deploy();
    contracts.push(contract);
    promises.push(contract.deployTransaction.wait(1));
    // because nonce should increase in sequence
    await sleep(1);
  }

  await Promise.all(promises);
  return contracts;
};

const generateWallets = (size) => {
  const wallets = [];
  for (let i = 0; i < size; i++) {
    const wallet = ethers.Wallet.createRandom();
    wallets.push(wallet);
  }
  return wallets;
};

const generateSignatures = (msgHash, wallets) => {
  let signatures = '0x';
  for (let i = 0; i < wallets.length; i++) {
    const wallet = wallets[i];
    const { v, r, s } = ecsign(
      Buffer.from(msgHash.slice(2), 'hex'),
      Buffer.from(wallet.privateKey.slice(2), 'hex')
    );
    const sigHex = toRpcSig(v, r, s);
    signatures += sigHex.slice(2);
  }
  return signatures;
};

const runErrorCase = async (txPromise, expectErrorMsg) => {
  try {
    await txPromise;
  } catch (e) {
    const error = e.error ? e.error.toString() : e.toString();
    expect(error.indexOf(expectErrorMsg) > -1).to.eq(true);
  }
};

const getMsgHashForSetNewCkbSpv = (
  DOMAIN_SEPARATOR,
  typeHash,
  newSpvAddress,
  nonce
) => {
  return keccak256(
    solidityPack(
      ['bytes1', 'bytes1', 'bytes32', 'bytes32'],
      [
        '0x19',
        '0x01',
        DOMAIN_SEPARATOR,
        keccak256(
          defaultAbiCoder.encode(
            ['bytes32', 'address', 'uint256'],
            [typeHash, newSpvAddress, nonce]
          )
        ),
      ]
    )
  );
};

const getMsgHashForAddHeaders = (DOMAIN_SEPARATOR, typeHash, headersData) => {
  return keccak256(
    solidityPack(
      ['bytes1', 'bytes1', 'bytes32', 'bytes32'],
      [
        '0x19',
        '0x01',
        DOMAIN_SEPARATOR,
        keccak256(
          defaultAbiCoder.encode(['bytes32', 'bytes'], [typeHash, headersData])
        ),
      ]
    )
  );
};

const { log } = console;

module.exports = {
  sleep,
  log,
  waitingForReceipt,
  deployContract,
  deployAll,
  generateWallets,
  generateSignatures,
  runErrorCase,
  getMsgHashForSetNewCkbSpv,
  getMsgHashForAddHeaders,
};
