const fs = require("fs");
const path = require("path");
const utils = require("@nervosnetwork/ckb-sdk-utils");
const log = console.log;
const ETH_RECIPIENT_TYPESCRIPT_PATH = path.join(
  __dirname,
  "../../../ckb-contracts/build/release/eth-recipient-typescript"
);

function blake2b(buffer) {
  return utils
    .blake2b(32, null, null, utils.PERSONAL)
    .update(buffer)
    .digest("binary");
}

async function main() {
  //ERC20
  const ERC20 = await ethers.getContractFactory(
    "contracts/test/ERC20.sol:ERC20"
  );
  const ERC20Deploy = await ERC20.deploy();
  await ERC20Deploy.deployed();
  const ERC20DeployAddr = ERC20Deploy.address;
  log("ERC20 deployed to:", ERC20DeployAddr);

  // deploy CKBChin
  const CKBChain = await ethers.getContractFactory(
    "contracts/CKBChain.sol:CKBChain"
  );
  const CKBChinDeploy = await CKBChain.deploy();
  await CKBChinDeploy.deployed();
  const CKBChinDeployAddr = CKBChinDeploy.address;
  log("CKBChin deployed to:", CKBChinDeployAddr);

  // deploy TokenLocker
  const bin = fs.readFileSync(ETH_RECIPIENT_TYPESCRIPT_PATH);
  const recipientCodeHash = utils.bytesToHex(blake2b(bin));
  log("recipientCodeHash", recipientCodeHash);
  const TokenLocker = await ethers.getContractFactory(
    "contracts/TokenLocker.sol:TokenLocker"
  );
  const locker = await TokenLocker.deploy(
    CKBChinDeployAddr,
    1,
    recipientCodeHash,
    0
  );
  await locker.deployed();
  const lockerAddr = locker.address;
  console.log("locker deployed to:", lockerAddr);

  const address = {
    ERC20Deploy: ERC20DeployAddr,
    CKBChainDeploy: CKBChinDeployAddr,
    TokenLockerDeploy: lockerAddr,
  };

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
