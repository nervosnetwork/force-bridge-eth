const {sleep} = require("./method");
const {NODE_URL,LUMOS_DB} = require("./config");
const { Indexer} = require('@ckb-lumos/indexer')
const axios = require('axios')


function deleteAll(path) {
    var files = [];
    if(fs.existsSync(path)) {
        files = fs.readdirSync(path);
        files.forEach(function(file, index) {
            var curPath = path + "/" + file;
            if(fs.statSync(curPath).isDirectory()) { // recurse
                deleteall(curPath);
            } else { // delete file
                fs.unlinkSync(curPath);
            }
        });
        fs.rmdirSync(path);
    }
};

const getTipBlockNumber = async () => axios({
    method: 'post',
    url: NODE_URL,
    data: {
        id: 1,
        jsonrpc: '2.0',
        method: 'get_tip_block_number',
        params: [],
    },
});

const waitForIndexing = async(indexer_client, target_stop, timeout) => {
    const { data: { result: nodeTipBlockNumber } } = await getTipBlockNumber();
    console.log("node current BlockNumber is: ", nodeTipBlockNumber);
    if (!indexer_client.running()) {
        indexer_client.startForever();
    }

    const startedAt = Date.now();
    while (true) {

        const currentTip = await indexer_client.tip();
        if (!currentTip) {
            continue;
        }
        if (target_stop) {
            if (BigInt(currentTip.block_number) >= BigInt(nodeTipBlockNumber)) {
                console.log("currentTip is: ", currentTip);
                break;
            }
        } else {
            console.log("currentTip is: ", currentTip);
        }

        // if (Date.now() - startedAt > timeout) {
        //     console.error("currentTip is: ", currentTip,"waiting for indexing is timeout");
        //     // throw new Error('waiting for indexing is timeout');
        // }
        await sleep(2000)
    }
}
async function main() {
    const indexer = new Indexer(NODE_URL, LUMOS_DB)
    await waitForIndexing( indexer,false,4* 60 * 1000)
}

// main();


module.exports= {
    waitForIndexing,
    deleteAll,
    getTipBlockNumber
}
