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

    // deploy TestBlake2b
    const factory = await ethers.getContractFactory("TestBlake2b");
    const contract = await factory.deploy();
    await contract.deployed();
    const contractAddr = contract.address;
    console.log("TestBlake2b deployed to:", contractAddr);

    // transfer 2 address
    const [owner, addr1] = await ethers.getSigners();

    let provider = ethers.getDefaultProvider()
    let transaction = {
        to: "0x627306090abab3a6e1400e9345bc60c78a8bef57",
        value: "0x20000000000000"
    };

    // Send the transaction
    console.log(await owner.sendTransaction(transaction))
    console.log(await owner.getBalance())
    console.log(await owner.transfer(transaction.to, transaction.value))
    console.log(await owner.getBalance())


    // Send the transaction
    // console.log(await owner.sendTransaction(transaction))

    // calc blake2b
    let res = await contract.ckbBlake2b("0x39e33c8ad2e7e4eb71610d2bcdfbb0cb0fde2f96418256914ad2f5be1d6e9331", "0x385dfb0153a0e3aec760120c4e333a4a6bec91eeaca359ef714709588d23ca16")
    assert(res === "0x93a9faceb827e8a431217f0e5fc6068c14cc62ac4cf73752ed4e9135adc364c8", `${res} !== 0x93a9faceb827e8a431217f0e5fc6068c14cc62ac4cf73752ed4e9135adc364c8`)
    console.log("ckbBlake2b res: ", res)

}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch(error => {
        console.error(error);
        process.exit(1);
    });
