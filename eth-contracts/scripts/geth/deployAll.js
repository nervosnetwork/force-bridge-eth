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
    console.log("waiting for block confirmations, about 1 minute");
    await sleep(60 * 1000);

    const contractBalance = await ERC20Deploy.callStatic.balanceOf("0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2");

    console.log(`tokenLocker contract erc20 balance: ${contractBalance.toString()}`);


    // deploy CKBChin
    const CKBChain = await ethers.getContractFactory(
        "contracts/CKBChain.sol:CKBChain"
    );
    const CKBChinDeploy = await CKBChain.deploy();
    await CKBChinDeploy.deployed();
    const CKBChinDeployAddr = CKBChinDeploy.address;
    console.log("CKBChin deployed to:", CKBChinDeployAddr);
    console.log("waiting for block confirmations, about 1 minute");
    await sleep(60 * 1000);

    const finalizedGcThreshold = 500;
    const canonicalGcThreshold = 40000;

    const initHeaderData ="0x0000000000000820a9daa6d47501000036100000000000000200004605080700b3f696ffbda4feb72731b797a135c5cc779200acea4aa45cac7df07e856403f350f98d988b232747c7f757a4b48b733754d063101934fe69a4128b5ad25855a300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de4a62557a97a32ef49cf85a678723000df76dee6b82000000e9fbaead700a0741571b602446aa2a2135177404285c15";
    const initBlockHash = "0x94e9522537032ef62fd21043ae1b1247738ffe264b16392ea5e2fe752376b5e0";
    let res = await CKBChinDeploy.initWithHeader(
        initHeaderData,
        initBlockHash,
        finalizedGcThreshold,
        canonicalGcThreshold
    );
    console.log("initHeader :", res.hash);
    console.log("waiting for block confirmations, about 1 minute");
    await sleep(60 * 1000);


    // deploy Eaglesong
    const Eaglesong = await ethers.getContractFactory(
        "contracts/Eaglesong.sol:Eaglesong"
    );
    const EaglesongDeploy = await Eaglesong.deploy();
    await EaglesongDeploy.deployed();
    const EaglesongDeployAddr = EaglesongDeploy.address;
    console.log("Eaglesong deployed to:", EaglesongDeployAddr);
    console.log("waiting for block confirmations, about 1 minute");
    await sleep(60 * 1000);


    // deploy Migrations
    const Migrations = await ethers.getContractFactory(
        "contracts/Migrations.sol:Migrations"
    );
    const MigrationsDeploy = await Migrations.deploy();
    await MigrationsDeploy.deployed();
    const MigrationsDeployAddr = MigrationsDeploy.address;
    console.log("Migrations deployed to:", MigrationsDeployAddr);
    console.log("waiting for block confirmations, about 1 minute");
    await sleep(60 * 1000);


    // deploy TokenLocker
    const TokenLocker = await ethers.getContractFactory(
        "contracts/TokenLocker.sol:TokenLocker"
    );
    const locker = await TokenLocker.deploy(CKBChinDeployAddr,1,"0x17fb928d15bf98064304f2126f6b387e33d0c3c6572d293143ca373929ec3b5c",0);
    await locker.deployed();
    const lockerAddr = locker.address;
    console.log("locker deployed to:", lockerAddr);
    console.log("waiting for block confirmations, about 1 minute");
    await sleep(60 * 1000);

    const fs = require('fs');
    // create a JSON object
    const address = {
        "ERC20Deploy": ERC20DeployAddr,
        "CKBChainDeploy":EaglesongDeployAddr,
        "EaglesongDeploy": EaglesongDeployAddr,
        "MigrationsDeploy":MigrationsDeployAddr,
        "TokenLockerDepoly":lockerAddr,
    };

    // convert JSON object to string
    const data = JSON.stringify(address);

    // write JSON string to a file
    fs.writeFile('address.json', data, (err) => {
        if (err) {
            throw err;
        }
        console.log("JSON data is saved.");
    });
    await sleep(60 * 1000);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
