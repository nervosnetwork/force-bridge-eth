const { log, waitingForReceipt, deployContract } = require('../../test/utils');
const { getHeaderAndHash, getHeadersVecAndHashes } = require('./generateData');

const deployAndInitWithGc = async (factoryPath) => {
  // 1. deploy CKBChain
  const contract = await deployContract(factoryPath);
  const provider = contract.provider;

  // 2. initWithHeader
  const finalizedGcThreshold = 40;
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

  // 3. add 40 headers for gc test
  let startIndex = 1;
  let size = 40;
  const [headers, _hashes] = getHeadersVecAndHashes(startIndex, size);
  startIndex += size;
  res = await contract.addHeaders(headers);
  const receipt = await waitingForReceipt(provider, res);
  log(
    `Init for Gc test, add ${size} Headers gas: ${
      receipt.gasUsed
    }, gas cost per header: ${receipt.gasUsed / size}`
  );

  return {
    contract,
    startIndex,
  };
};

const benchmarkWithGc = async (factoryPath) => {
  // 1. deploy and init
  let { contract, startIndex } = await deployAndInitWithGc(factoryPath);

  // 2. benchmark
  let reportSize = [1, 2, 3, 4, 5, 10, 20, 30, 40];
  for (let size of reportSize) {
    const [headers, _] = getHeadersVecAndHashes(startIndex, size);
    startIndex += size;
    let res = await contract.addHeaders(headers);
    const receipt = await res.wait(1);
    log(
      `add ${size} Headers gas: ${receipt.gasUsed}, gas cost per header: ${
        receipt.gasUsed / size
      }`
    );

    // check if addHeaders success
    let actualTipNumber = await contract.callStatic.latestBlockNumber();
    log(`current tipBlockNumber: ${actualTipNumber}\r\n`);
  }
};

const main = async () => {
  // addHeaders with Blake2b calc blockHash
  log(`---------------addHeaders with Blake2b calc blockHash`);
  await benchmarkWithGc('contracts/CKBChain.sol:CKBChain');
  log(`---------------end\r\n\r\n`);
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
