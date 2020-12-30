const fs = require('fs');
const TOML = require('@iarna/toml');
const { deployContract, deployAll, sleep } = require('../../test/utils');

async function main() {
  const forceConfigPath = process.env.FORCE_CONFIG_PATH;
  if (!forceConfigPath) {
    throw 'FORCE_CONFIG_PATH not set';
  }
  const forceConfig = TOML.parse(fs.readFileSync(forceConfigPath));
  const bridge_lockscript_code_hash = forceConfig.bridge_lockscript.code_hash;
  const recipient_typescript_code_hash =
    forceConfig.recipient_typescript.code_hash;

  // deploy erc20 tokens, CKBChain
  const factoryPaths = [
    'contracts/test/ERC20.sol:DAI',
    'contracts/test/ERC20.sol:USDT',
    'contracts/test/ERC20.sol:USDC',
    'contracts/CKBChain.sol:CKBChain',
  ];
  const contracts = await deployAll(factoryPaths);
  const [DAIAddr, USDTAddr, USDCAddr, CKBChinDeployAddr] = contracts.map(
    (contract) => contract.address
  );

  // deploy TokenLocker
  const locker = await deployContract(
    'contracts/TokenLocker.sol:TokenLocker',
    CKBChinDeployAddr,
    1,
    '0x' + recipient_typescript_code_hash,
    0,
    '0x' + bridge_lockscript_code_hash
  );
  const lockerAddr = locker.address;

  console.error(`
  DAIAddr: ${DAIAddr}, USDTAddr: ${USDTAddr}, USDCAddr: ${USDCAddr}
  ERC20Addr: ${DAIAddr}
  CKBChin deploy to: ${CKBChinDeployAddr}
  locker deploy to: ${lockerAddr}
  `);

  const address = {
    erc20: DAIAddr,
    dai: DAIAddr,
    usdt: USDTAddr,
    usdc: USDCAddr,
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
  console.error('write eth addr into settings successfully');

  const tokenLockerJson = require('../../artifacts/contracts/TokenLocker.sol/TokenLocker.json');
  const lockerABI = tokenLockerJson.abi;
  const ckbChainJSON = require('../../artifacts/contracts/CKBChain.sol/CKBChain.json');
  const ckbChainABI = ckbChainJSON.abi;
  fs.writeFileSync(
    '../offchain-modules/lib/src/util/token_locker_abi.json',
    JSON.stringify(lockerABI, null, 2)
  );
  fs.writeFileSync(
    '../offchain-modules/lib/src/util/ckb_chain_abi.json',
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
