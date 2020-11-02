const ethers = require("ethers");

function sleep(s) {
  return new Promise((resolve) => setTimeout(resolve, 1000 * s));
}

async function main() {
    let provider = ethers.getDefaultProvider("http://localhost:8545")
    let retry_count = 0;
    let time_count = 0
    while (true) {
        let blockNumber;
        try {
            blockNumber = await provider.getBlockNumber()
        } catch (e) {
            if (retry_count < 3) {
                retry_count++
                console.log(`network error: ${e}. retry ${retry_count} times`)
                await sleep(3)
                break
            } else {
                throw e
            }
        }
        if (blockNumber > 0) {
            console.log(`miner is ready after ${time_count} seconds`)
            return
        }
        console.log(`miner not ready(generating dag), ${time_count} seconds passed`)
        time_count += 5
        await sleep(5)
    }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
