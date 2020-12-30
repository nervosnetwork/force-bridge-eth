const origin = require('./data/testHeaderHashes.json');
const { log } = console;
const BN = require('bn.js');
const headers = [];
const blockHashes = [];

const init = () => {
  origin.map((obj) => {
    headers.push(obj.input);
    blockHashes.push(obj.output);
  });
};
init();

const fixedLength = (str, targetLen = 8) => {
  const len = str.length;
  return '0'.repeat(targetLen - len) + str;
};

const fixedLengthLe = (str, targetLen = 8) => {
  const len = str.length;
  return str + '0'.repeat(targetLen - len);
};

/*
 * @notice             generate a param of CKBChain.initWithHeader()
 * @param index        index of testHeaders
 * @return             [Header as molecule bytes, BlockHash as molecule bytes]
 * */
const getHeaderAndHash = (index) => {
  return [headers[index], blockHashes[index]];
};
/*
 * @notice             generate a param of CKBChain.addHeaders()
 * @param size         length of headerVec
 * @return             [HeaderVec as molecule bytes, BlockHashes as molecule bytes]
 * */
const getHeadersVecAndHashes = (startIndex, size) => {
  const sizeBn = new BN(size);
  const buf = sizeBn.toBuffer();
  const leHexSize = buf.reverse().toString('hex');
  let headerHex = fixedLengthLe(leHexSize, 8);
  headers.slice(startIndex, startIndex + size).map((headerStr) => {
    headerHex += headerStr.slice(2);
  });

  // let hashHex = ''
  // blockHashes.slice(startIndex, startIndex + size).map(
  //     (item) => {
  //         hashHex += item.slice(2)
  //     }
  // )
  return ['0x' + headerHex, blockHashes.slice(startIndex, startIndex + size)];
};

const getTinyHeaders = (startIndex, size) => {
  const tinyHeaders = [];
  headers.slice(startIndex, startIndex + size).forEach((headerStr, index) => {
    let tinyHeader = '0x';
    // number
    const numberIndex = 2 + 16 * 2;
    tinyHeader += headerStr.slice(numberIndex, numberIndex + 8 * 2);

    // blockHash
    tinyHeader += blockHashes[index].slice(2);

    const txRootIndex = 2 + 64 * 2;
    // txRoot
    tinyHeader += headerStr.slice(txRootIndex, txRootIndex + 32 * 2);

    tinyHeaders.push(tinyHeader);
  });
  return tinyHeaders;
};

module.exports = {
  getHeaderAndHash,
  getHeadersVecAndHashes,
  getTinyHeaders,
};
