const { log, waitingForReceipt, deployContract } = require('../../test/utils');
const { getHeaderAndHash, getHeadersVecAndHashes } = require('./generateData');

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
  const provider = contract.provider;

  // 2. benchmark
  let startIndex = 1;
  let reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40];
  for (let size of reportSize) {
    const [headers, _hashes] = getHeadersVecAndHashes(startIndex, size);
    startIndex += size;
    let res = await contract.addHeaders(headers);
    const receipt = await waitingForReceipt(provider, res);
    console.log(
      `add ${size} Headers gas: ${receipt.gasUsed}, gas cost per header: ${
        receipt.gasUsed / size
      }`
    );
  }
};

const main = async () => {
  // addHeaders with Blake2b calc blockHash
  console.log(`---------------addHeaders with Blake2b calc blockHash`);
  await benchmark('contracts/CKBChain.sol:CKBChain');
  console.log(`---------------end\r\n\r\n`);

  // addHeaders without Blake2b
  // console.log(`---------------addHeaders without Blake2b`);
  // await benchmark("contracts/CKBChain.sol:CKBChain")
  // console.log(`---------------end\r\n\r\n`);
};

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
