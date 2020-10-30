const ethers = require("ethers");

function sleep(s) {
    return new Promise(resolve => setTimeout(resolve, 1000*s));
}

async function main() {
    let provider = ethers.getDefaultProvider("http://localhost:8545")
    let count = 0
    while (true) {
        let blockNumber = await provider.getBlockNumber()
        if (blockNumber>0) {
            console.log(`miner is ready after ${count} seconds`)
            return
        }
        console.log(`miner not ready(generating dag), ${count} seconds passed`)
        count += 5
        await sleep(5)
    }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error(error);
        process.exit(1);
    });