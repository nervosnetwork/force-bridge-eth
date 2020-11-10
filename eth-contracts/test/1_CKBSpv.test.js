const { expect } = require("chai");
const { log, waitingForReceipt } = require("./utils");
const vectors = require("./data/testSpv.json");

const {
  extractBlockNumber,
  extractBlockHash,
  expectedTransactionsRoot,
} = vectors;

contract("CKBSpv", () => {
  let ckbChain;
  let provider;

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

  describe("proveTxExist correct case", async function () {
    // disable timeout
    this.timeout(0);
    it("Should proveTxExist verified", async () => {
      for (let i = 0; i < expectedTransactionsRoot.length; i++) {
        const transactionsRoot = expectedTransactionsRoot[i].output;
        const blockHash = extractBlockHash[i].output;
        const blockNumber = parseInt(extractBlockNumber[i].output);
        const latestBlockNumber = blockNumber + 100;

        // mockForProveTxExist
        const res = await ckbChain.mockForProveTxExist(
          latestBlockNumber,
          blockNumber,
          blockHash,
          transactionsRoot
        );
        await waitingForReceipt(provider, res);
        expect(await ckbChain.callStatic.latestBlockNumber()).to.equal(
          latestBlockNumber
        );

        // proveTxExist
        const txProof = expectedTransactionsRoot[i].input;
        const proveRes = await ckbChain.proveTxExist(txProof, 100);
        const receipt = await waitingForReceipt(provider, res);
        log(`proveTxExist gasUsed: ${receipt.gasUsed.toString()}`);
        expect(proveRes).to.equal(true);
      }
    });
  });
});
