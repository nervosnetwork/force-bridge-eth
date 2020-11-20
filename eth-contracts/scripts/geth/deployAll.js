const fs = require("fs");
const TOML = require("@iarna/toml");

async function main() {
  const forceConfigPath = process.env.FORCE_CONFIG_PATH;
  if (!forceConfigPath) {
    throw "FORCE_CONFIG_PATH not set";
  }
  const forceConfig = TOML.parse(fs.readFileSync(forceConfigPath));
  const bridge_lockscript_code_hash = forceConfig.bridge_lockscript.code_hash;
  const recipient_typescript_code_hash =
    forceConfig.recipient_typescript.code_hash;

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

  // deploy TokenLocker
  const TokenLocker = await ethers.getContractFactory(
    "contracts/TokenLocker.sol:TokenLocker"
  );
  const locker = await TokenLocker.deploy(
    CKBChinDeployAddr,
    1,
    "0x" + recipient_typescript_code_hash,
    0,
    "0x" + bridge_lockscript_code_hash
  );
  await locker.deployed();
  const lockerAddr = locker.address;
  console.error("locker deployed to:", lockerAddr);

  const address = {
    erc20: ERC20DeployAddr,
    ckbChain: CKBChinDeployAddr,
    tokenLocker: lockerAddr,
  };
  const data = JSON.stringify(address);
  console.log(data);

  // write eth address to settings
  forceConfig.eth_token_locker_addr = lockerAddr;
  forceConfig.eth_ckb_chain_addr = CKBChinDeployAddr;
  const new_config = TOML.stringify(forceConfig);
  fs.writeFileSync(forceConfigPath, new_config);
  console.error("write eth addr into settings successfully");
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
