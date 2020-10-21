// We require the Buidler Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
// When running the script with `buidler run <script>` you'll find the Buidler
// Runtime Environment's members available in the global scope.
const bre = require("@nomiclabs/buidler");

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
    // Buidler always runs the compile task when running scripts through it.
    // If this runs in a standalone fashion you may want to call compile manually
    // to make sure everything is compiled
    // await bre.run('compile');

    // deploy MockCKBSpv
    const MockCKBSpv = await ethers.getContractFactory("MockCKBSpv");
    const mockSpv = await MockCKBSpv.deploy();
    await mockSpv.deployed();
    const mockSpvAddr = mockSpv.address;
    console.log("mockSpv deployed to:", mockSpvAddr);

    // deploy TokenLocker
    const TokenLocker = await ethers.getContractFactory("TokenLocker");
    const locker = await TokenLocker.deploy(mockSpvAddr, 20);
    await locker.deployed();
    const lockerAddr = locker.address;
    console.log("locker deployed to:", lockerAddr);
    console.log("waiting for block confirmations, about 1 minute")
    await sleep(60 * 1000);

    // lockETH 0.123
    let amount = ethers.utils.parseEther("0.123");
    let res = await locker.lockETH(amount, "0.123 ether just for lockETH test", {value: amount})
    console.log("lockETH res: ", res)
    console.log("waiting for block confirmations, about 1 minute")
    await sleep(60 * 1000);

    // unlockETH
    res = await locker.unlockToken([0])
    console.log("unlockETH res: ", res)

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error(error);
        process.exit(1);
    });
