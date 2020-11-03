/* global artifacts contract describe before it assert */
const BN = require("bn.js");

/* eslint-disable-next-line no-unresolved */
const vectors = require("./data/testSpv.json");

const ViewSpv = artifacts.require("ViewSpvTest");

const {
  extractTxMerkleIndex,
  extractBlockNumber,
  extractBlockHash,
  extractTxHash,
  extractWitnessesRoot,
  extractLemmas,
} = vectors;

contract("ViewSpv", () => {
  let instance;
  before(async function () {
    // disable timeout
    this.timeout(0);
    instance = await ViewSpv.new();
  });

  describe("#txMerkleIndex", async () => {
    it("extracts the txMerkleIndex from a CKBTxProof", async () => {
      for (let i = 0; i < extractTxMerkleIndex.length; i += 1) {
        const res = await instance.txMerkleIndex(extractTxMerkleIndex[i].input);
        const expected = new BN(extractTxMerkleIndex[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe("#blockNumber", async () => {
    it("extracts the blockNumber from a CKBTxProof", async () => {
      for (let i = 0; i < extractBlockNumber.length; i += 1) {
        const res = await instance.blockNumber(extractBlockNumber[i].input);
        const expected = new BN(extractBlockNumber[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe("#blockHash", async () => {
    it("extracts the blockHash from a CKBTxProof", async () => {
      for (let i = 0; i < extractBlockHash.length; i += 1) {
        const res = await instance.blockHash(extractBlockHash[i].input);
        assert.strictEqual(extractBlockHash[i].output, res);
      }
    });
  });

  describe("#txHash", async () => {
    it("extracts the txHash from a CKBTxProof", async () => {
      for (let i = 0; i < extractTxHash.length; i += 1) {
        const res = await instance.txHash(extractTxHash[i].input);
        assert.strictEqual(extractTxHash[i].output, res);
      }
    });
  });

  describe("#witnessesRoot", async () => {
    it("extracts the witnessesRoot from a CKBTxProof", async () => {
      for (let i = 0; i < extractWitnessesRoot.length; i += 1) {
        const res = await instance.witnessesRoot(extractWitnessesRoot[i].input);
        assert.strictEqual(extractWitnessesRoot[i].output, res);
      }
    });
  });

  describe("#lemmas", async () => {
    it("extracts the lemmas from a CKBTxProof", async () => {
      for (let i = 0; i < extractLemmas.length; i += 1) {
        const res = await instance.lemmas(extractLemmas[i].input);
        if (extractLemmas[i].output === "0x") {
          extractLemmas[i].output = null;
        }
        assert.strictEqual(extractLemmas[i].output, res);
      }
    });
  });
});
