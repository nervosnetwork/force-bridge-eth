const { ecsign, toRpcSig } = require('ethereumjs-util');
const { blake2b, PERSONAL } = require('@nervosnetwork/ckb-sdk-utils');
const { keccak256, defaultAbiCoder, solidityPack } = ethers.utils;
const BN = require('bn.js');

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
  const factory = await ethers.getContractFactory(factoryPath);
  const contract = await factory.deploy(...args);
  await contract.deployTransaction.wait(1);
  return contract;
};

const deployContractByWallet = async (wallet, factoryPath, ...args) => {
  const factory = await ethers.getContractFactory(factoryPath, wallet);
  const contract = await factory.deploy(...args);
  await contract.deployTransaction.wait(1);
  return contract;
};

const deployUpgradableContractFirstTime = async (
  factoryPathStorage,
  factoryPathLogic,
  _proxy_admin,
  ...storageArgs
) => {
  storageArgs.push(_proxy_admin);
  const storageContract = await deployContract(
    factoryPathStorage,
    ...storageArgs
  );
  const logicContract = await deployContract(factoryPathLogic);

  const txRes = await storageContract.sysAddDelegates([logicContract.address], {
    from: _proxy_admin,
  });
  await txRes.wait(1);

  const instance = await ethers.getContractAt(
    factoryPathLogic,
    storageContract.address
  );

  log(`${instance.address}`);

  return instance;
};

const deployUpgradableContractFirstTimeByWallet = async (
  wallet,
  factoryPathStorage,
  factoryPathLogic,
  _proxy_admin,
  ...storageArgs
) => {
  storageArgs.push(_proxy_admin);
  const storageContract = await deployContractByWallet(
    wallet,
    factoryPathStorage,
    ...storageArgs
  );
  const logicContract = await deployContractByWallet(wallet, factoryPathLogic);

  const txRes = await storageContract.sysAddDelegates([logicContract.address], {
    from: _proxy_admin,
    gasLimit: 1000000,
  });
  await txRes.wait(1);

  const instance = await ethers.getContractAt(
    factoryPathLogic,
    storageContract.address,
    wallet
  );

  log(`${instance.address}`);

  return instance;
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
    //expect(error.indexOf(expectErrorMsg) > -1).to.eq(true);
    expect(error).to.have.string(expectErrorMsg);
  }
};

const retryPromise = async (txPromise, times) => {
  let res = null;
  for (let i = 0; i < times; i++) {
    try {
      res = await txPromise;
      return res;
    } catch (e) {
      log(`send tx failed, retry ${i}`, e);
      await sleep(2);
    }
  }
  return res;
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
          defaultAbiCoder.encode(
            ['bytes32', 'bytes[]'],
            [typeHash, headersData]
          )
        ),
      ]
    )
  );
};

const getMsgHashForAddHistoryTxRoot = (
  DOMAIN_SEPARATOR,
  typeHash,
  initBlockNumber,
  latestBlockNumber,
  historyTxRoot
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
            ['bytes32', 'uint64', 'uint64', 'bytes32'],
            [typeHash, initBlockNumber, latestBlockNumber, historyTxRoot]
          )
        ),
      ]
    )
  );
};

const ckbBlake2b = (hexStr) => {
  let str = hexStr.startsWith('0x') ? hexStr.slice(2) : hexStr;
  const instance = blake2b(32, null, null, PERSONAL);
  const input = new Uint8Array(Buffer.from(str, 'hex'));
  instance.update(input);
  return '0x' + instance.digest('hex');
};

const fixedLengthLe = (str, targetLen = 8) => {
  const len = str.length;
  return str + '0'.repeat(targetLen - len);
};

const clear0x = (hexStr) => {
  return hexStr.startsWith('0x') ? hexStr.slice(2) : hexStr;
};

const getMockTinyHeaderParam = (
  blockNumber,
  blockHash,
  txRoot = 'f'.repeat(64)
) => {
  let tinyHeaderHex = '0x';

  // 1. number
  const numberBN = new BN(blockNumber);
  const buf = numberBN.toBuffer();
  const leHexNumber = buf.reverse().toString('hex');
  tinyHeaderHex += fixedLengthLe(leHexNumber, 16);

  // 2. blockHash
  tinyHeaderHex += clear0x(blockHash);

  // 3. txRoot
  tinyHeaderHex += clear0x(txRoot);

  return [tinyHeaderHex];
};

const { log } = console;

module.exports = {
  sleep,
  log,
  waitingForReceipt,
  deployContract,
  deployAll,
  deployContractByWallet,
  deployUpgradableContractFirstTime,
  deployUpgradableContractFirstTimeByWallet,
  generateWallets,
  generateSignatures,
  runErrorCase,
  getMsgHashForSetNewCkbSpv,
  getMsgHashForAddHeaders,
  getMsgHashForAddHistoryTxRoot,
  ckbBlake2b,
  retryPromise,
  fixedLengthLe,
  getMockTinyHeaderParam,
  clear0x,
};
