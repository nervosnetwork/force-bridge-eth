/* global artifacts contract describe before it assert */
const BN = require('bn.js');

/* eslint-disable-next-line no-unresolved */
const spvVectors = require('./data/testSpv.json');
const historyTxRootVectors = require('./data/testViewHistoryTxRoot.json');
const historyTxProofVectors = require('./data/testHistoryTxProof.json');

const ViewSpv = artifacts.require('ViewSpvTest');

const {
  extractTxMerkleIndex,
  extractBlockNumber,
  extractBlockHash,
  extractTxHash,
  extractWitnessesRoot,
  extractLemmas,
} = spvVectors;

const {
  extractInitBlockNumber,
  extractLatestBlockNumber,
  extractIndices,
  extractProofLeaves,
  extractLemmas: extractTxRootLemmas,
} = historyTxRootVectors;

const {
  extractBlockNumber: extractTxBlockNumber,
  extractTxMerkleIndex: extractHistoryTxMerkleIndex,
  extractWitnessesRoot: extractHistoryWitnessesRoot,
  extractLemmas: extractHistoryLemmas,
  extractRawTransaction,
  calcTxHash,
} = historyTxProofVectors;

contract('ViewSpv', () => {
  let instance;
  before(async function () {
    // disable timeout
    this.timeout(0);
    instance = await ViewSpv.new();
  });

  // CKBHistoryTxProof
  describe('#txBlockNumber', async () => {
    it('extracts the blockNumber from a CKBHistoryTxProof', async () => {
      for (let i = 0; i < extractTxBlockNumber.length; i += 1) {
        const res = await instance.txBlockNumber(extractTxBlockNumber[i].input);
        const expected = new BN(extractTxBlockNumber[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#historyTxMerkleIndex', async () => {
    it('extracts the txMerkleIndex from a CKBHistoryTxProof', async () => {
      for (let i = 0; i < extractHistoryTxMerkleIndex.length; i += 1) {
        const res = await instance.historyTxMerkleIndex(
          extractHistoryTxMerkleIndex[i].input
        );
        const expected = new BN(extractHistoryTxMerkleIndex[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#historyWitnessesRoot', async () => {
    it('extracts the witnessesRoot from a CKBHistoryTxProof', async () => {
      for (let i = 0; i < extractHistoryWitnessesRoot.length; i += 1) {
        const res = await instance.historyWitnessesRoot(
          extractHistoryWitnessesRoot[i].input
        );
        assert.strictEqual(extractHistoryWitnessesRoot[i].output, res);
      }
    });
  });

  describe('#historyLemmas', async () => {
    it('extracts the lemmas from a CKBHistoryTxProof', async () => {
      for (let i = 0; i < extractHistoryLemmas.length; i += 1) {
        const res = await instance.historyLemmas(extractHistoryLemmas[i].input);
        if (extractHistoryLemmas[i].output === '0x') {
          extractHistoryLemmas[i].output = null;
        }
        assert.strictEqual(extractHistoryLemmas[i].output, res);
      }
    });
  });

  describe('#rawTransaction', async () => {
    it('extracts the rawTransaction from a CKBHistoryTxProof', async () => {
      for (let i = 0; i < extractRawTransaction.length; i += 1) {
        const res = await instance.rawTransaction(
          extractRawTransaction[i].input
        );
        if (extractRawTransaction[i].output === '0x') {
          extractRawTransaction[i].output = null;
        }
        assert.strictEqual(extractRawTransaction[i].output, res);
      }
    });
  });

  // CKBHistoryTxRootProof
  describe('#indices', async () => {
    it('extracts the indices from a CKBHistoryTxRootProof', async () => {
      for (let i = 0; i < extractIndices.length; i += 1) {
        const res = await instance.indices(extractIndices[i].input);
        if (extractIndices[i].output === '0x') {
          extractIndices[i].output = null;
        }
        assert.strictEqual(extractIndices[i].output, res);
      }
    });
  });

  describe('#proofLeaves', async () => {
    it('extracts the proofLeaves from a CKBHistoryTxRootProof', async () => {
      for (let i = 0; i < extractProofLeaves.length; i += 1) {
        const res = await instance.proofLeaves(extractProofLeaves[i].input);
        if (extractProofLeaves[i].output === '0x') {
          extractProofLeaves[i].output = null;
        }
        assert.strictEqual(extractProofLeaves[i].output, res);
      }
    });
  });

  describe('#txRootLemmas', async () => {
    it('extracts the lemmas from a CKBHistoryTxRootProof', async () => {
      for (let i = 0; i < extractTxRootLemmas.length; i += 1) {
        const res = await instance.txRootLemmas(extractTxRootLemmas[i].input);
        if (extractTxRootLemmas[i].output === '0x') {
          extractTxRootLemmas[i].output = null;
        }
        assert.strictEqual(extractTxRootLemmas[i].output, res);
      }
    });
  });
});
