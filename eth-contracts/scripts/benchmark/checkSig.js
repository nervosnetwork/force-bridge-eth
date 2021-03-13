const { log, waitingForReceipt, deployContract } = require('../../test/utils');
const { getHeaderAndHash, getHeadersVecAndHashes } = require('./generateData');
const { ecsign } = require('ethereumjs-util');

const { hexlify } = ethers.utils;

const deployAndInit = async (factoryPath) => {
  // 1. deploy CKBChain
  const contract = await deployContract(factoryPath);
  const provider = contract.provider;

  // 2. initWithHeader
  const finalizedGcThreshold = 500;
  const canonicalGcThreshold = 40000;

  const [initHeaderData, initBlockHash] = getHeaderAndHash(0);
  log(`initHeaderData: ${initHeaderData}, initBlockHash: ${initBlockHash}`);
  let res = await contract.initWithHeader(
    initHeaderData,
    initBlockHash,
    finalizedGcThreshold,
    canonicalGcThreshold
  );
  let txReceipt = await waitingForReceipt(provider, res);
  log(`initWithHeader gasUsed: ${txReceipt.gasUsed.toString()}`);

  return contract;
};

const benchmark = async (factoryPath) => {
  // 1. deploy and init
  const contract = await deployAndInit(factoryPath);
  const wallet = ethers.Wallet.createRandom();
  log(`sk: ${wallet.privateKey}`);
  log(`addr: ${wallet.address}`);

  // 2. benchmark checkSig
  const digest =
    '0x0000000000000000000000000000000000000000000000000000000000001234';
  const { v, r, s } = ecsign(
    Buffer.from(digest.slice(2), 'hex'),
    Buffer.from(wallet.privateKey.slice(2), 'hex')
  );
  let reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40];
  for (let size of reportSize) {
    let gasUsed = await contract.callStatic.checkSig(
      wallet.address,
      v,
      hexlify(r),
      hexlify(s),
      size
    );
    log(
      `checkSig ${size} times, gas: ${gasUsed}, gas cost per checkSig: ${
        gasUsed / size
      }`
    );
  }
};

const main = async () => {
  // addHeaders with Blake2b calc blockHash
  log(`---------------checkSig`);
  await benchmark('contracts/CKBChainV2.sol:CKBChainV2');
  log(`---------------end\r\n\r\n`);

  // addHeaders without Blake2b
  // log(`---------------addHeaders without Blake2b`);
  // await benchmark("contracts/CKBChain.sol:CKBChain")
  // log(`---------------end\r\n\r\n`);
};

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
