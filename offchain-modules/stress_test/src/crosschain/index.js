const {SDCollector} = require("./ckb_indexer");
const axios = require('axios')
const CKB = require('@nervosnetwork/ckb-sdk-core').default;
const Web3 = require('web3')
const { Indexer, CellCollector } = require('@ckb-lumos/indexer')
const {getOrCreateBridgeCell,
    placeCrossChainOrder,
    sleep,getLockStatus,
    getCrosschainHistory,
    getSudtBalance,
    getBestBlockHeight,
    initToken,
    getBurnStatus,
    prepareERCToken} = require("./method");
const {
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
    TokenLockerAddress,
    ETH_TOKEN_ADDRESS,
} = require("./config");

const fs = require('fs');
const {waitForIndexing,deleteAll} = require("./ckb_lumos");


const ckb = new CKB(NODE_URL);
const web3 = new Web3(ETH_NODE_URL);

// const userPWEthLockHash = ckb.utils.scriptToHash(userPWEthLock);
// console.log("userPWEthLockHash: ", userPWEthLockHash);

// const userEthCKBAddress = ckb.utils.fullPayloadToAddress({
//     args: userPWEthLockHash,
//     type: ORDERBOOK_LOCK_TYPE == "type" ? ckb.utils.AddressType.TypeCodeHash : ckb.utils.AddressType.DataCodeHash,
//     prefix: ckb.utils.AddressPrefix.Testnet,
//     codeHash: ORDERBOOK_LOCK_CODEHASH,
// })

const generateWallets = (size) => {
    const privkeys = [];
    for (let i = 0; i < size; i++) {
        const wallet = web3.eth.accounts.create();
        privkeys.push(wallet.privateKey)
    }
    return privkeys;
}

const prepareBridgeCells = async (privkeys,cellNum,tokenAddress) => {
    let createFutures = [];
    for (let i = 0; i < privkeys.length; i++) {
        const addr = ckb.utils.privateKeyToAddress(privkeys[i], {prefix: 'ckt'})

        let createFut = getOrCreateBridgeCell(addr, tokenAddress, bridgeFee, cellNum);
        createFutures.push(createFut);
    }
    const createOutpoints = await Promise.all(createFutures);
    console.log("create bridge outpoints",createOutpoints.map((res) => { return res.data.outpoints ;}))
}


const prepareAccounts = async (fromPrivkey, toPrivkeys) => {
    const fromAddress = ckb.utils.privateKeyToAddress(fromPrivkey, {prefix: 'ckt'})
    const fromPublicKey = ckb.utils.privateKeyToPublicKey(fromPrivkey)
    const fromPublicKeyHash = `0x${ckb.utils.blake160(fromPublicKey, 'hex')}`
    console.log("rich ckb account : ",fromAddress)
    await ckb.loadDeps()
    let lock =    {
        codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
        hashType: "type",
        args: fromPublicKeyHash,
    }

    /* lomus indexer which is useless*/
    // const indexer = new Indexer(NODE_URL, LUMOS_DB)
    // await waitForIndexing( indexer,true,4* 60 * 1000)
    // const unspentCells = await ckb.loadCells({ indexer, CellCollector, lock })
    // indexer.stop()
    // deleteAll(lumos_db_tmp)

    const ckb_collect =  new SDCollector()
    const unspentCells = await ckb_collect.getCells(fromPublicKeyHash);
    console.log("unspentCells",unspentCells)

    let liveCells = []
    for (let i = 0; i < unspentCells.length; i++) {
        let res = await ckb.rpc.getLiveCell(unspentCells[i].outPoint,false);
        console.log("cell capacity: ",res.cell.output.capacity, " cell status: ", res.status)
        if(res.status === 'live') {
            liveCells.push(unspentCells[i])
        }
    }
    console.log("liveCells",liveCells)

    let tx = ckb.generateRawTransaction({
        fromAddress,
        toAddress: "ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37",
        capacity: BigInt(15000_00000000),
        fee: BigInt(100000),
        safeMode: true,
        cells: liveCells,
        deps: [ckb.config.secp256k1Dep],
    });
    let restCapacity = BigInt(0);
    for (let i = 0; i < tx.outputs.length; i++) {
        restCapacity = BigInt(tx.outputs[i].capacity) + restCapacity
    }
    console.log("restCapacity: ",restCapacity)
    tx.outputs.splice(0,   tx.outputs.length);
    tx.outputsData.splice(0,   tx.outputsData.length);
    let capacity = BigInt( 1000_00000000);
    for (let i = 0; i < toPrivkeys.length; i++) {
        // const addr = ckb.utils.privateKeyToAddress(toPrivkeys[i], {prefix: 'ckt'})
        const publicKey = ckb.utils.privateKeyToPublicKey(toPrivkeys[i])
        const publicKeyHash = `0x${ckb.utils.blake160(publicKey, 'hex')}`
        let output = {
            capacity: "0x"+capacity.toString(16),
            lock:  {
                codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                hashType: "type",
                args: publicKeyHash
            },
            type: null,
        };
        tx.outputs.push(output)
        tx.outputsData.push("0x");
    }
    let output = {
        capacity: "0x"+(restCapacity - capacity * BigInt(toPrivkeys.length)).toString(16),
        lock:  {
            codeHash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
            hashType: "type",
            args: fromPublicKeyHash
        },
        type: null,
    };
    tx.outputs.push(output)
    tx.outputsData.push("0x");
    tx.witnesses = tx.inputs.map((_, i) => (i > 0 ? '0x' : {
        lock: '',
        inputType: '',
        outputType: '',
    }));
    const signedTx = ckb.signTransaction(fromPrivkey)(tx)
    // console.log(JSON.stringify(signedTx, null, 2))
    const txHash = await ckb.rpc.sendTransaction(signedTx)
    console.log("prepare account tx hash",txHash)
}



const lockToken = async (recipientCKBAddress, cellNum,crossToken, lockTxSender, nonce) => {
    // bridge has been created
    // let get_res = await getOrCreateBridgeCell(recipientCKBAddress, crossToken.tokenAddress, bridgeFee, cellNum);
    // let bridgeCells = [...get_res.data.outpoints];
    // console.log("bridgeCells",bridgeCells);

    const gasPrice = await web3.eth.getGasPrice()
    const send_with_outpoint = async (index) => {
        const txFromBridge = await placeCrossChainOrder(index, "", crossToken.udtDecimal, recipientCKBAddress, crossToken.orderPrice, crossToken.orderAmount, isBid, crossToken.tokenAddress, bridgeFee, gasPrice, lockTxSender,nonce + index);
        const res = await web3.eth.accounts.signTransaction(txFromBridge.data, RichETHPrivkey);
        const rawTX = res.rawTransaction;
        const receipt = await web3.eth.sendSignedTransaction(rawTX);
        if (!receipt.status) {
            console.error("failed to lock tx hash : ",txHash)
        }
        // await sleep( 90 * 1000);
        let txHash =receipt.transactionHash;

        if(txHash.indexOf("0x") === 0){
            txHash = txHash.substring(2)
        }
        await getLockStatus(txHash)
        return txHash;
    }

    let futures = [];
    for (let index = 0; index < cellNum; index++) {
        let fut = send_with_outpoint(index);
        futures.push(fut);
    }
    const crosschainTxHashes = await Promise.all(futures);
    // console.log("lock hashes ", crosschainTxHashes);

    return crosschainTxHashes;
}

const batchMintToken = async (crosschainTxHashes) => {
    await Promise.all(crosschainTxHashes.map(txHash => relayEthToCKB(txHash)));
}



const burnToken = async (privkey, txFee, unlockFee, amount, tokenAddress, recipientAddress) => {
    const ckb_client = new CKB(NODE_URL);
    const addr = ckb_client.utils.privateKeyToAddress(privkey, {prefix: 'ckt'})

    const postData = {
        from_lockscript_addr: addr,
        tx_fee: txFee,
        unlock_fee: unlockFee,
        amount: amount,
        token_address: tokenAddress,
        recipient_address: recipientAddress,
    }

    console.log("burn postData: ", JSON.stringify(postData))
    let res;
    while(res === "" || res === undefined || res == null) {
        try{
            res = await axios.post(`${FORCE_BRIDGER_SERVER_URL}/burn`, postData, {timeout: 1000 * 60 * 5})
        } catch(error){
            console.error("failed to post burn interface: ",error.response.status,error.response.statusText)
        }
        await sleep(10*1000)
    }
    const rawTx = ckb_client.rpc.resultFormatter.toTransaction(res.data.raw_tx)

    rawTx.witnesses = rawTx.inputs.map((_, i) => (i > 0 ? '0x' : {
        lock: '',
        inputType: '',
        outputType: '',
    }));

    const signedTx = ckb_client.signTransaction(privkey)(rawTx)
    delete signedTx.hash
    let txHash = await ckb_client.rpc.sendTransaction(signedTx)
    if(txHash.indexOf("0x") === 0){
        txHash = txHash.substring(2)
    }
    await getBurnStatus(txHash)
    return txHash
}

const batchLockToken = async (burnPrivkeys,crossToken) => {
    console.log("*************************  start lock ", crossToken.tokenAddress, "      ********************************");
    let richETHAddr = web3.eth.accounts.privateKeyToAccount(RichETHPrivkey).address
    let nonce = await web3.eth.getTransactionCount(richETHAddr)
    console.log("rich eth account", richETHAddr, "start nonce :", nonce);
    const cellNum = 1
    let lockFutures = [];
    for (let i = 0; i < burnPrivkeys.length; i++) {
        const addr = ckb.utils.privateKeyToAddress(burnPrivkeys[i], {prefix: 'ckt'})
        let lockFut = lockToken(addr, cellNum, crossToken,richETHAddr,nonce);
        lockFutures.push(lockFut);
        nonce = nonce + cellNum;
    }
    const lockHashes = await Promise.all(lockFutures);
    console.log("lock hashes ", lockHashes);
    // await batchMintToken(waitMintTxs);
    // wait relay the lock tx proof to CKB
    console.log("***************************  end lock ", crossToken.tokenAddress, "      ********************************");
}


const recycleRecipientCell = async (privkey) => {
    const ckb_client = new CKB(NODE_URL);
    const addr = ckb_client.utils.privateKeyToAddress(privkey, {prefix: 'ckt'})

    const postData = {
        from_lockscript_addr: addr,
        tx_fee: burnTxFee,
    }

    console.log("recycle_recipient postData: ", JSON.stringify(postData))
    let res;
    while(res === "" || res === undefined || res == null) {
        try{
            res = await axios.post(`${FORCE_BRIDGER_SERVER_URL}/recycle_recipient`, postData, {timeout: 1000 * 60 * 5})
        } catch(error){
            console.error("failed to post recycle_recipient interface: ",error.response.status,error.response.statusText)
        }
        await sleep(10*1000)
    }
    const rawTx = ckb_client.rpc.resultFormatter.toTransaction(res.data.raw_tx)

    rawTx.witnesses = rawTx.inputs.map((_, i) => (i > 0 ? '0x' : {
        lock: '',
        inputType: '',
        outputType: '',
    }));

    const signedTx = ckb_client.signTransaction(privkey)(rawTx)
    delete signedTx.hash
    // console.log("recycle_recipient tx",JSON.stringify(signedTx, null, 2))
    let txHash = await ckb_client.rpc.sendTransaction(signedTx)
    if(txHash.indexOf("0x") === 0){
        txHash = txHash.substring(2)
    }
    return txHash
}

const batchRecycleRecipientCell = async(burnPrivkeys) => {
    console.log("************************    start batchRecycleRecipientCell    ***********************************");
    let recycleRecipientFutures = [];
    for (let i = 0; i < burnPrivkeys.length; i++) {
        let recycleRecipientFut = recycleRecipientCell(burnPrivkeys[i]);
        recycleRecipientFutures.push(recycleRecipientFut);
    }
    const recycleRecipientHashes = await Promise.all(recycleRecipientFutures);
    console.log("recycle recipient cell hashes ", recycleRecipientHashes);
    console.log("********************* end batchRecycleRecipientCell   ********************************");

}

const batchBurnToken = async (burnPrivkeys,crossToken) => {

    console.log("**************************   start query ", crossToken.tokenAddress, " balance ********************************");
    for (let i = 0; i < burnPrivkeys.length; i++) {
        const addr = ckb.utils.privateKeyToAddress(burnPrivkeys[i], {prefix: 'ckt'})
        let balance = await getSudtBalance(addr, crossToken.tokenAddress)
    }

    console.log("************************     start burn ", crossToken.tokenAddress, "   ***********************************");
    let burnFutures = [];
    for (let i = 0; i < burnPrivkeys.length; i++) {
        let burnFut = burnToken(burnPrivkeys[i], burnTxFee, unlockFee, unlockAmount, crossToken.tokenAddress, recipientETHAddress);
        burnFutures.push(burnFut);
    }
    const burnHashes = await Promise.all(burnFutures);
    console.log("burn hashes ", burnHashes);
    console.log("********************* end burn ", crossToken.tokenAddress, " and start test interface    ********************************");

    await getBestBlockHeight()
    await getCrosschainHistory(recipientETHAddress.toLowerCase())

    console.log("***********************************   end test interface    ********************************");
}


const crossChain = async (burnPrivkeys, crossToken) => {
    await prepareAccounts(RichCKBPrivkey,burnPrivkeys)
    await initToken(crossToken.tokenAddress)
    await batchLockToken(burnPrivkeys,crossToken)
    await batchBurnToken(burnPrivkeys,crossToken)
    await batchRecycleRecipientCell(burnPrivkeys)
}


async function main() {
    const concurrency_number = 2
    const cross_chain_tokens = ["ETH"]
    const burnPrivkeys = generateWallets(concurrency_number);
    fs.writeFileSync(
        'burnPrivkeys',
        JSON.stringify(burnPrivkeys, null, 2)
    );
    console.log("generate ckb keys for receive sudt ",burnPrivkeys)

    let tokens_map = prepareERCToken()

    for (let i = 0; i < cross_chain_tokens.length; i++) {
        let cross_token = tokens_map.get(cross_chain_tokens[i])
        await crossChain(burnPrivkeys, cross_token)
    }
}

main();
