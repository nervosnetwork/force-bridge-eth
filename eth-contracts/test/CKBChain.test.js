const { expect } = require("chai");
const { log, waitingForGasUsed, sleep } = require("./utils");
const vectors = require("./data/testSpv.json");

const {
  extractBlockNumber,
  extractBlockHash,
  expectedTransactionsRoot,
} = vectors;

contract("CKBChain", () => {
  let ckbChain, provider;

  before(async function () {
    // disable timeout
    this.timeout(0);
    const factory = await ethers.getContractFactory(
      "contracts/CKBChain.sol:CKBChain"
    );
    ckbChain = await factory.deploy();
    await ckbChain.deployed();
    provider = ckbChain.provider;
  });

  describe("initWithHeader correct case", async function () {
    // disable timeout
    this.timeout(0);
    it("Should initWithHeader success", async () => {
      // const headerData = '0x000000007ea9081ab0ccd0786e01000057040000000000000000005704cf0600258cb71cd81aeaca2e8141549322b85a5a83f1b3728d1bd860b4877453eba67b4313d9603ff24ef9197375a912562b88a94d7324c946195b9f8309763ef5c251937616899b34cc24a55012fe60cd46699df23d90ed35c5c49098dc55568c1d2b0000000000000000000000000000000000000000000000000000000000000000522e520916b1a12e161de18a0d872300a6e80f65451e000000efb310fe3dff0676030000312b000000000000216033d2';
      // const expectPowMsg = '0xcbecbaf6a2deee59b2eab3bbae5388128ce9f30183336526c9081419f163fc6076030000312b000000000000216033d2';
      // const res = await verifier._addHeader(headerData)
      // log('powMessage res = ', res)

      const initHeaderData = '0x000000007ea9081a0867f5706e01000005000000000000000000000500cf060083832d6367429901a4bf763a6d6cbdc658a2624a8a4cda7427edd6fad65d0f7d8877c8cab9d920c4ce87c67661ffc566ffe34d5c1ec7341ad53a3d91b90c22960000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030bba492fd1ea12e17d1fb8ef2862300ae897b092b00000000710b00c0fefe0635a9e31900000000000000558c23d5ab'
      const initBlockHash = '0x2567f226c73b04a6cb3ef04b3bb10ab99f37850794cd9569be7de00bac4db875'
      const finalizedGcThreshold = 500;
      const canonicalGcThreshold = 40000;
      let res = await ckbChain.initWithHeader(initHeaderData, initBlockHash, finalizedGcThreshold, canonicalGcThreshold);
      let gasUsed = await waitingForGasUsed(provider, res);
      log(`initWithHeader gasUsed: ${gasUsed.toString()}`);
      // await sleep(10)

      // check if init success
      let expectTipNumber = 5
      let actualTipNumber = await ckbChain.callStatic.getLatestBlockNumber();
      expect(actualTipNumber).to.equal(expectTipNumber);

      let expectCanonicalHeaderHash = initBlockHash
      let actualCanonicalHeaderHash = await ckbChain.callStatic.getCanonicalHeaderHash(expectTipNumber);
      log(`actualCanonicalHeaderHash: ${actualCanonicalHeaderHash}`)

    });
  });
});
