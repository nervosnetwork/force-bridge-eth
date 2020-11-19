// We require the Buidler Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
// When running the script with `buidler run <script>` you'll find the Buidler
// Runtime Environment's members available in the global scope.

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
    // Buidler always runs the compile task when running scripts through it.
    // If this runs in a standalone fashion you may want to call compile manually
    // to make sure everything is compiled
    // await bre.run('compile');

    //ERC20
    const ERC20 = await ethers.getContractFactory(
        "contracts/test/ERC20.sol:ERC20"
    );
    const ERC20Deploy = await ERC20.deploy();
    await ERC20Deploy.deployed();
    const ERC20DeployAddr = ERC20Deploy.address;
    console.log("ERC20 deployed to:", ERC20DeployAddr);

    // deploy CKBChin
    const CKBChain = await ethers.getContractFactory(
        "contracts/CKBChain.sol:CKBChain"
    );
    const CKBChinDeploy = await CKBChain.deploy();
    await CKBChinDeploy.deployed();
    const CKBChinDeployAddr = CKBChinDeploy.address;
    console.log("CKBChin deployed to:", CKBChinDeployAddr);

    // deploy Eaglesong
    const Eaglesong = await ethers.getContractFactory(
        "contracts/Eaglesong.sol:Eaglesong"
    );
    const EaglesongDeploy = await Eaglesong.deploy();
    await EaglesongDeploy.deployed();
    const EaglesongDeployAddr = EaglesongDeploy.address;
    console.log("Eaglesong deployed to:", EaglesongDeployAddr);

    const receipent_code_hash =
        "0xa170baee8a38fcc33a83a51db412a51b74101e931f7f90586de1971b11154ad4";
    const bridge_code_hash =
        "0xa5ee819012157f00d71b6ff305db7d8ed94705c0f3b90bb911116dd6968a8a2d";
    // deploy TokenLocker
    const TokenLocker = await ethers.getContractFactory(
        "contracts/TokenLocker.sol:TokenLocker"
    );
    const locker = await TokenLocker.deploy(
        CKBChinDeployAddr,
        1,
        receipent_code_hash,
        0,
        bridge_code_hash
    );
    await locker.deployed();
    const lockerAddr = locker.address;
    console.log("locker deployed to:", lockerAddr);

    const address = {
        ERC20Deploy: ERC20DeployAddr,
        CKBChainDeploy: CKBChinDeployAddr,
        EaglesongDeploy: EaglesongDeployAddr,
        TokenLockerDepoly: lockerAddr,
    };
    const fs = require("fs");
    const data = JSON.stringify(address);
    fs.writeFileSync("./scripts/geth/address.json", data);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
