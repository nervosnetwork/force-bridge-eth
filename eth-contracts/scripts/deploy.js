const fs = require("fs");
const TOML = require("@iarna/toml");
const { sleep } = require("../test/utils");

async function main() {
  const forceConfigPath = process.env.FORCE_CONFIG_PATH;
  const network = process.env.FORCE_NETWORK;
  if (!forceConfigPath) {
    throw "FORCE_CONFIG_PATH not set";
  }
  const forceConfig = TOML.parse(fs.readFileSync(forceConfigPath));
  let network_config;
  if (network) {
    network_config = forceConfig.networks_config[network];
  } else {
    network_config = forceConfig.networks_config[forceConfig.default_network];
  }
  const provider = new ethers.providers.JsonRpcProvider(
    network_config.ethereum_rpc_url
  );
  const deployedContracts = forceConfig.deployed_contracts;
  const bridge_lockscript_code_hash =
    deployedContracts.bridge_lockscript.code_hash;
  const recipient_typescript_code_hash =
    deployedContracts.recipient_typescript.code_hash;
  const wallet = new ethers.Wallet(
    "0x" + network_config.ethereum_private_keys[0],
    provider
  );

  const contractPaths = [
    "contracts/test/ERC20.sol:DAI",
    "contracts/test/ERC20.sol:USDT",
    "contracts/test/ERC20.sol:USDC",
    "contracts/CKBChain.sol:CKBChain",
  ];

  const contracts = [];
  const promises = [];
  for (const path of contractPaths) {
    const factory = await ethers.getContractFactory(path, wallet);
    const contract = await factory.deploy();
    contracts.push(contract);
    promises.push(contract.deployTransaction.wait(1));
    // because nonce should increase in sequence
    await sleep(1);
  }
  await Promise.all(promises);
  const [DAIAddr, USDTAddr, USDCAddr, CKBChainAddr] = contracts.map(
    (contract) => contract.address
  );

  console.error(`
    DAIAddr: ${DAIAddr}, USDTAddr: ${USDTAddr}, USDCAddr: ${USDCAddr}, 
    CKBChinAddr: ${CKBChainAddr}
  `);

  // deploy TokenLocker
  let TokenLocker = await ethers.getContractFactory(
    "contracts/TokenLocker.sol:TokenLocker",
    wallet
  );
  const locker = await TokenLocker.deploy(
    CKBChainAddr,
    1,
    "0x" + recipient_typescript_code_hash,
    0,
    "0x" + bridge_lockscript_code_hash
  );
  await locker.deployed();
  const lockerAddr = locker.address;
  console.error("tokenLocker", lockerAddr);

  const address = {
    daiContractAddr: DAIAddr,
    usdtContractAddr: USDTAddr,
    usdcContractAddr: USDCAddr,
  };
  const data = JSON.stringify(address);
  console.log(data);

  // write eth address to settings
  deployedContracts.eth_token_locker_addr = lockerAddr;
  deployedContracts.eth_ckb_chain_addr = CKBChainAddr;
  const new_config = TOML.stringify(forceConfig);
  fs.writeFileSync(forceConfigPath, new_config);
  console.error("write eth addr into config successfully");

  const tokenLockerJson = require("../artifacts/contracts/TokenLocker.sol/TokenLocker.json");
  const lockerABI = tokenLockerJson.abi;
  const ckbChainJSON = require("../artifacts/contracts/CKBChain.sol/CKBChain.json");
  const ckbChainABI = ckbChainJSON.abi;
  fs.writeFileSync(
    "../offchain-modules/lib/src/util/token_locker_abi.json",
    JSON.stringify(lockerABI, null, 2)
  );
  fs.writeFileSync(
    "../offchain-modules/lib/src/util/ckb_chain_abi.json",
    JSON.stringify(ckbChainABI, null, 2)
  );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
