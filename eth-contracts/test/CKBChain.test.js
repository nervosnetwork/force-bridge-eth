const { expect } = require("chai");
const { log, waitingForTxReceipt, sleep } = require("./utils");
const vectors = require("./data/testVectors.json");

const {
  extractBlockNumber,
  calculateBlockHash,
  extractTransactionsRoot,
  extractEpoch,
  indexHeaderVec
} = vectors;

contract("CKBChain", () => {
  let ckbChain, provider, initHeaderIndex;

  before(async function () {
    // disable timeout
    this.timeout(0);
    const factory = await ethers.getContractFactory(
      "contracts/CKBChain.sol:CKBChain"
    );
    ckbChain = await factory.deploy();
    await ckbChain.deployed();
    provider = ckbChain.provider;
    initHeaderIndex = extractBlockNumber.length - 3;  // it will add 2 headers
  });

  describe("initWithHeader correct case", async function () {
    // disable timeout
    this.timeout(0);
    it("Should initWithHeader success", async () => {
      const finalizedGcThreshold = 500;
      const canonicalGcThreshold = 40000;

      const initHeaderData = calculateBlockHash[initHeaderIndex].input
      const initBlockHash = calculateBlockHash[initHeaderIndex].output
      let res = await ckbChain.initWithHeader(initHeaderData, initBlockHash, finalizedGcThreshold, canonicalGcThreshold);
      let txReceipt = await waitingForTxReceipt(provider, res);
      log(`initWithHeader gasUsed: ${txReceipt.gasUsed.toString()}`);

      // verify result
      let expectTipNumber = extractBlockNumber[initHeaderIndex].output
      let actualTipNumber = await ckbChain.callStatic.getLatestBlockNumber();
      expect(actualTipNumber).to.equal(expectTipNumber);

      let expectCanonicalHeaderHash = initBlockHash
      let actualCanonicalHeaderHash = await ckbChain.callStatic.getCanonicalHeaderHash(expectTipNumber);
      expect(actualCanonicalHeaderHash).to.equal(expectCanonicalHeaderHash);

      let expectLatestEpoch = extractEpoch[initHeaderIndex].output
      let actualLatestEpoch = await ckbChain.callStatic.getLatestEpoch();
      expect(actualLatestEpoch).to.equal(expectLatestEpoch);

      let expectTransactionsRoot = extractTransactionsRoot[initHeaderIndex].output
      let actualTransactionsRoot = await ckbChain.callStatic.getCanonicalTransactionsRoot(initBlockHash);
      expect(actualTransactionsRoot).to.equal(expectTransactionsRoot);
    });

    it("Should addHeaders success", async () => {
      const startIndex = initHeaderIndex + 1;  // add headers that follow initHeader

      // addHeaders
      const headersInput = indexHeaderVec[startIndex].input
      let res = await ckbChain.addHeaders(headersInput)
      let txReceipt = await waitingForTxReceipt(provider, res);
      log(JSON.stringify(txReceipt.logs, null, 4));

      // verify result
      const headers = indexHeaderVec[startIndex].output;
      const endHeaderIndex = startIndex + headers.length - 1;
      let expectTipNumber = extractBlockNumber[ endHeaderIndex ].output
      let actualTipNumber = await ckbChain.callStatic.getLatestBlockNumber();
      expect(actualTipNumber).to.equal(expectTipNumber);

      for (let i = 0; i < headers.length; i++) {
        const headerIndex = startIndex + i;
        let expectBlockHash = calculateBlockHash[headerIndex].output
        let blockNumber = extractBlockNumber[headerIndex].output
        let actualBlockHash = await ckbChain.callStatic.getCanonicalHeaderHash(blockNumber);
        expect(actualBlockHash).to.equal(expectBlockHash);
      }
    });

  })
})
