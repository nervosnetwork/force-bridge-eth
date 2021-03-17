/* local config */
const ETH_NODE_URL= 'http://127.0.0.1:8545'
const FORCE_BRIDGER_SERVER_URL = 'http://127.0.0.1:3003' //update to your force server url
const CKB_INDEXER_URL= 'http://127.0.0.1:8116'
const NODE_URL = 'http://127.0.0.1:8114/' //update to your node url
const RichCKBPrivkey = "0xa6b023fec4fc492c23c0e999ab03b01a6ca5524a3560725887a8de4362f9c9cc";
const RichETHPrivkey = '0xc4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a' //update with your own private key
const recipientETHAddress = '0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2'//orig; bob:'0xBeB7C1d39B59DF17613F82AF0EC265565414d608'


const ETH_TOKEN_ADDRESS = '0x0000000000000000000000000000000000000000'
// Test config
const DAI_TOKEN_ADDRESS = '0xC4401D8D5F05B958e6f1b884560F649CdDfD9615'
const USDT_TOKEN_ADDRESS = '0x1cf98d2a2f5b0BFc365EAb6Ae1913C275bE2618F'
const USDC_TOKEN_ADDRESS = '0x1F0D2251f51b88FaFc90f06F7022FF8d82154B1a'
const TokenLockerAddress = '0x4347818B33aaf0b442A977900585B9ad1e1B581F'

// lock params
const bridgeFee = '0x0'
const isBid = false;

// unlock params
const unlockFee = "0x1"
const unlockAmount = "0x2"
const burnTxFee = "0.1"


const ORDERBOOK_LOCK_CODEHASH = '0x279bee9fa98959029766c0e0ce19cd91b7180fd15b600a9e95140149b524c53b'
const ORDERBOOK_LOCK_TYPE = 'type'
const PW_LOCK_CODEHASH = '0x58c5f491aba6d61678b7cf7edf4910b1f5e00ec0cde2f42e0abb4fd9aff25a63'
const PW_LOCK_HASHTYPE = 'type'


const userPWEthLock = {
    codeHash: PW_LOCK_CODEHASH,
    hashType: PW_LOCK_HASHTYPE,
    args: recipientETHAddress,
};

const lumos_db_tmp = "lumos_db_tmp/"
// const path = require('path')
// const LUMOS_DB = path.join(lumos_db_tmp, 'lumos_db')
const LUMOS_DB = ""

module.exports = {
    ETH_NODE_URL,
    FORCE_BRIDGER_SERVER_URL,
    NODE_URL,
    RichETHPrivkey,
    userPWEthLock,
    bridgeFee,
    isBid,
    unlockFee,
    unlockAmount,
    burnTxFee,
    ORDERBOOK_LOCK_CODEHASH,
    ORDERBOOK_LOCK_TYPE,
    recipientETHAddress,
    RichCKBPrivkey,
    lumos_db_tmp,
    LUMOS_DB,
    CKB_INDEXER_URL,
    DAI_TOKEN_ADDRESS,
    USDT_TOKEN_ADDRESS,
    USDC_TOKEN_ADDRESS,
    ETH_TOKEN_ADDRESS,
    TokenLockerAddress,
}
