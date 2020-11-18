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

  // deploy Migrations
  const Migrations = await ethers.getContractFactory(
    "contracts/Migrations.sol:Migrations"
  );
  const MigrationsDeploy = await Migrations.deploy();
  await MigrationsDeploy.deployed();
  const MigrationsDeployAddr = MigrationsDeploy.address;
  console.log("Migrations deployed to:", MigrationsDeployAddr);

  const receipent_code_hash =
    "0x17fb928d15bf98064304f2126f6b387e33d0c3c6572d293143ca373929ec3b5c";
  // deploy TokenLocker
  const TokenLocker = await ethers.getContractFactory(
    "contracts/TokenLocker.sol:TokenLocker"
  );
  const locker = await TokenLocker.deploy(
    CKBChinDeployAddr,
    1,
    receipent_code_hash,
    0
  );
  await locker.deployed();
  const lockerAddr = locker.address;
  console.log("locker deployed to:", lockerAddr);

  const fs = require("fs");
  const address = {
    ERC20Deploy: ERC20DeployAddr,
    CKBChainDeploy: CKBChinDeployAddr,
    EaglesongDeploy: EaglesongDeployAddr,
    MigrationsDeploy: MigrationsDeployAddr,
    TokenLockerDepoly: lockerAddr,
  };

  const data = JSON.stringify(address);

  fs.writeFile("./scripts/geth/address.json", data, (err) => {
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
