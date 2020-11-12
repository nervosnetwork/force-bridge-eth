const origin = require("./data/testHeaderHashes.json");
const { log } = console;
const BN = require("bn.js");
const headers = [];
const blockHashes = [];

const init = () => {
  origin.map((obj) => {
    headers.push(obj.input);
    blockHashes.push(obj.output);
  });
};
init();

function fixedLength(str, targetLen = 8) {
  const len = str.length;
  return "0".repeat(targetLen - len) + str;
}

function fixedLengthLe(str, targetLen = 8) {
  const len = str.length;
  return str + "0".repeat(targetLen - len);
}

/*
 * @notice             generate a param of CKBChain.initWithHeader()
 * @param index        index of testHeaders
 * @return             [Header as molecule bytes, BlockHash as molecule bytes]
 * */
function getHeaderAndHash(index) {
  return [headers[index], blockHashes[index]];
}
/*
 * @notice             generate a param of CKBChain.addHeaders()
 * @param size         length of headerVec
 * @return             [HeaderVec as molecule bytes, BlockHashes as molecule bytes]
 * */
function getHeadersVecAndHashes(startIndex, size) {
  const sizeBn = new BN(size);
  const buf = sizeBn.toBuffer();
  const leHexSize = buf.reverse().toString("hex");
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
  return ["0x" + headerHex, blockHashes.slice(startIndex, startIndex + size)];
}

module.exports = {
  getHeaderAndHash,
  getHeadersVecAndHashes,
};
