const fs = require("fs");
const TOML = require('@iarna/toml');

const forceConfigPath = '/tmp/.force-bridge-cli/config.toml';
const forceConfig = TOML.parse(fs.readFileSync(forceConfigPath));
const bridge_lockscript_code_hash = forceConfig.bridge_lockscript.code_hash;
const recipient_typescript_code_hash = forceConfig.recipient_typescript.code_hash;

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
  console.error("ERC20 deployed to:", ERC20DeployAddr);

  // deploy CKBChin
  const CKBChain = await ethers.getContractFactory(
    "contracts/CKBChain.sol:CKBChain"
  );
  const CKBChinDeploy = await CKBChain.deploy();
  await CKBChinDeploy.deployed();
  const CKBChinDeployAddr = CKBChinDeploy.address;
  console.error("CKBChin deployed to:", CKBChinDeployAddr);

  // deploy Eaglesong
  const Eaglesong = await ethers.getContractFactory(
    "contracts/Eaglesong.sol:Eaglesong"
  );
  const EaglesongDeploy = await Eaglesong.deploy();
  await EaglesongDeploy.deployed();
  const EaglesongDeployAddr = EaglesongDeploy.address;
  console.error("Eaglesong deployed to:", EaglesongDeployAddr);

  // deploy TokenLocker
  const TokenLocker = await ethers.getContractFactory(
    "contracts/TokenLocker.sol:TokenLocker"
  );
  const locker = await TokenLocker.deploy(
    CKBChinDeployAddr,
    1,
    "0x" + recipient_typescript_code_hash,
    0
  );
  await locker.deployed();
  const lockerAddr = locker.address;
  console.error("locker deployed to:", lockerAddr);

  const address = {
    erc20: ERC20DeployAddr,
    ckb_chain: CKBChinDeployAddr,
    eaglesong: EaglesongDeployAddr,
    tokenLocker: lockerAddr,
  };
  const data = JSON.stringify(address);
  console.log(data);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
