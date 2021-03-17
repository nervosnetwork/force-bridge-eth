const BigNumber = require('bignumber.js')

/* eslint-disable no-bitwise */
const U128_MAX = BigInt(2) ** BigInt(128) - BigInt(1);
const U128_MIN = BigInt(0);

const writeBigUInt128LE = (u128) => {
    if (u128 < U128_MIN) {
        throw new Error(`u128 ${u128} too small`);
    }
    if (u128 > U128_MAX) {
        throw new Error(`u128 ${u128} too large`);
    }
    const buf = Buffer.alloc(16);
    buf.writeBigUInt64LE(u128 & BigInt('0xFFFFFFFFFFFFFFFF'), 0);
    buf.writeBigUInt64LE(u128 >> BigInt(64), 8);
    return `0x${buf.toString('hex')}`;
};

const readBigUInt128LE = (leHex) => {
    if (leHex.length !== 34 || !leHex.startsWith('0x')) {
        throw new Error('leHex format error');
    }
    const buf = Buffer.from(leHex.slice(2), 'hex');
    return (buf.readBigUInt64LE(8) << BigInt(64)) + buf.readBigUInt64LE(0);
};

const parseAmountFromSUDTData = (leHex) => {
    try {
        return readBigUInt128LE(leHex.startsWith('0x') ? leHex.slice(0, 34) : `0x${leHex.slice(0, 32)}`);
    } catch (error) {
        return BigInt(0);
    }
};

const toHexString = (str) => {
    return `0x${new BigNumber(str).toString(16)}`
};

module.exports = {
    writeBigUInt128LE,
    parseAmountFromSUDTData,
    toHexString,
};
