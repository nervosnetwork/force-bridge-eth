/* global artifacts contract describe before it assert */
const BN = require('bn.js');

/* eslint-disable-next-line no-unresolved */
const vectors = require('./data/testVectors.json');

const ViewCKB = artifacts.require('ViewCKBTest');

const {
  extractSince,
  extractPreviousOutput,
  extractCodeHash,
  extractHashType,
  extractArgs,

  extractRawHeader,
  extractNonce,

  extractVersion,
  extractCompactTarget,
  extractTimestamp,
  extractBlockNumber,
  extractEpoch,
  extractParentHash,
  extractTransactionsRoot,
  extractUnclesHash,
  indexHeaderVec,
} = vectors;

contract('ViewCKB', () => {
  let instance;
  before(async function () {
    // disable timeout
    this.timeout(0);
    instance = await ViewCKB.new();
  });

  describe('#since', async () => {
    it('extracts the since from a CellInput', async () => {
      for (let i = 0; i < extractSince.length; i += 1) {
        const res = await instance.since(extractSince[i].input);
        const expected = new BN(extractSince[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#previousOutput', async () => {
    it('extracts the outpoint from a CellInput', async () => {
      for (let i = 0; i < extractPreviousOutput.length; i += 1) {
        const res = await instance.previousOutput(
          extractPreviousOutput[i].input
        );
        assert.strictEqual(extractPreviousOutput[i].output, res);
      }
    });
  });

  describe('#codeHash', async () => {
    it('extracts the codeHash from a Scritp', async () => {
      for (let i = 0; i < extractCodeHash.length; i += 1) {
        const res = await instance.codeHash(extractCodeHash[i].input);
        assert.strictEqual(extractCodeHash[i].output, res);
      }
    });
  });

  describe('#hashType', async () => {
    it('extracts the hashType from a Scritp', async () => {
      for (let i = 0; i < extractHashType.length; i += 1) {
        const res = await instance.hashType(extractHashType[i].input);
        const expected = new BN(extractHashType[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#args', async () => {
    it('extracts the args from a Script', async () => {
      for (let i = 0; i < extractArgs.length; i += 1) {
        const res = await instance.args(extractArgs[i].input);

        // "0x" means non-data
        if (extractArgs[i].output === '0x') {
          extractArgs[i].output = null;
        }
        assert.strictEqual(extractArgs[i].output, res);
      }
    });
  });

  // members in Header
  describe('#rawHeader', async () => {
    it('extracts the rawHeader from a Header', async () => {
      for (let i = 0; i < extractRawHeader.length; i += 1) {
        const res = await instance.rawHeader(extractRawHeader[i].input);
        assert.strictEqual(extractRawHeader[i].output, res);
      }
    });
  });

  describe('#nonce', async () => {
    it('extracts the nonce from a Header', async () => {
      for (let i = 0; i < extractNonce.length; i += 1) {
        const res = await instance.nonce(extractNonce[i].input);
        const expected = new BN(extractNonce[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  // members in RawHeader
  describe('#version', async () => {
    it('extracts the version from a RawHeader', async () => {
      for (let i = 0; i < extractVersion.length; i += 1) {
        const res = await instance.version(extractVersion[i].input);
        const expected = new BN(extractVersion[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#compactTarget', async () => {
    it('extracts the compactTarget from a RawHeader', async () => {
      for (let i = 0; i < extractCompactTarget.length; i += 1) {
        const res = await instance.compactTarget(extractCompactTarget[i].input);
        const expected = new BN(extractCompactTarget[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#timestamp', async () => {
    it('extracts the timestamp from a RawHeader', async () => {
      for (let i = 0; i < extractTimestamp.length; i += 1) {
        const res = await instance.timestamp(extractTimestamp[i].input);
        const expected = new BN(extractTimestamp[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#blockNumber', async () => {
    it('extracts the blockNumber from a RawHeader', async () => {
      for (let i = 0; i < extractBlockNumber.length; i += 1) {
        const res = await instance.blockNumber(extractBlockNumber[i].input);
        const expected = new BN(extractBlockNumber[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#epoch', async () => {
    it('extracts the epoch from a RawHeader', async () => {
      for (let i = 0; i < extractEpoch.length; i += 1) {
        const res = await instance.epoch(extractEpoch[i].input);
        const expected = new BN(extractEpoch[i].output, 10);
        assert(res.eq(expected));
      }
    });
  });

  describe('#parentHash', async () => {
    it('extracts the parentHash from a RawHeader', async () => {
      for (let i = 0; i < extractParentHash.length; i += 1) {
        const res = await instance.parentHash(extractParentHash[i].input);
        assert.strictEqual(extractParentHash[i].output, res);
      }
    });
  });

  describe('#transactionsRoot', async () => {
    it('extracts the transactionsRoot from a RawHeader', async () => {
      for (let i = 0; i < extractTransactionsRoot.length; i += 1) {
        const res = await instance.transactionsRoot(
          extractTransactionsRoot[i].input
        );
        assert.strictEqual(extractTransactionsRoot[i].output, res);
      }
    });
  });

  describe('#unclesHash', async () => {
    it('extracts the unclesHash from a RawHeader', async () => {
      for (let i = 0; i < extractUnclesHash.length; i += 1) {
        const res = await instance.unclesHash(extractUnclesHash[i].input);
        assert.strictEqual(extractUnclesHash[i].output, res);
      }
    });
  });

  // get a header view from headerVec
  describe('#getHeader', async () => {
    it('get a header view from a HeaderVec', async () => {
      for (let i = 0; i < indexHeaderVec.length; i += 1) {
        const headerVec = indexHeaderVec[i].output;

        for (let index = 0; index < headerVec.length; index++) {
          const res = await instance.indexHeaderVec(
            indexHeaderVec[i].input,
            index
          );
          assert.strictEqual(headerVec[index], res);
        }
      }
    });
  });
});
