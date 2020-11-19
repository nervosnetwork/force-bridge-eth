const fs = require("fs");
const toml = require("toml");
// const utils = require("@nervosnetwork/ckb-sdk-utils");
const utils = require("../test/utils");

const forceConfigPath = "/tmp/.force-bridge-cli/config.toml";
const forceConfig = toml.parse(fs.readFileSync(forceConfigPath));
const bridge_lockscript_code_hash = forceConfig.bridge_lockscript.code_hash;
// console.log(bridge_lockscript_code_hash);

async function main() {
  let factory = await ethers.getContractFactory(
    "contracts/test/ERC20.sol:ERC20"
  );
  const erc20 = await factory.deploy();
  await erc20.deployed();
  console.error("erc20 deployed to:", erc20.address);

  factory = await ethers.getContractFactory(
    "contracts/test/MockCKBSpv.sol:MockCKBSpv"
  );
  const mockSpv = await factory.deploy();
  await mockSpv.deployed();

  factory = await ethers.getContractFactory(
    "contracts/TokenLocker.sol:TokenLocker"
  );
  const tokenLocker = await factory.deploy(
    mockSpv.address,
    123,
    "0x" + bridge_lockscript_code_hash,
    0
  );
  await tokenLocker.deployed();
  console.error("tokenLocker deployed to:", tokenLocker.address);
  const output = {
    tokenLocker: tokenLocker.address,
    erc20: erc20.address,
  };
  console.log(JSON.stringify(output));

  // lockETH 0.123
  // let provider = tokenLocker.provider;
  // let amount = ethers.utils.parseEther("0.123");
  // const res = await tokenLocker.lockETH(
  //     ethers.utils.parseEther("0.001"),
  //     "0x12345600",
  //     "0x12345611",
  //     "0x12345622",
  //     { value: amount }
  // );
  // // console.log("lockETH res: ", res);
  // const receipt = await utils.waitingForReceipt(provider, res);
  // console.log(`receipt:`, receipt);

  // unlockETH
  // res = await locker.unlockToken([0], [0]);
  // console.log("unlockETH res: ", res);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
