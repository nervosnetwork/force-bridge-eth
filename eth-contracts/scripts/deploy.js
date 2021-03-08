const fs = require('fs');
const TOML = require('@iarna/toml');
const EthUtil = require('ethereumjs-util');
const { upgrades } = require('hardhat');
const { sleep, ckbBlake2b, log } = require('../test/utils');

async function main() {
  const retryTimes = 20;
  for (let i = 0; i < retryTimes; i++) {
    try {
      await deploy();
      log(`deploy success!`);
      break;
    } catch (e) {
      log(e);
      log('retry deploy times: ', i);
    }
  }
}

async function deploy() {
  // get force config
  const forceConfigPath = process.env.FORCE_CONFIG_PATH;
  const network = process.env.FORCE_NETWORK;
  if (!forceConfigPath) {
    throw 'FORCE_CONFIG_PATH not set';
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
  let recipientCellTypescriptHashType =
    deployedContracts.recipient_typescript.hash_type;

  let lightClientTypescriptHash = ckbBlake2b(
    deployedContracts.light_client_cell_script.cell_script
  );
  console.error(`lightClientTypescriptHash: `, lightClientTypescriptHash);

  const wallet = new ethers.Wallet(
    '0x' + network_config.ethereum_private_keys[0],
    provider
  );
  const adminAddress = wallet.address;
  console.error(`adminAddress : `, adminAddress);

  // deploy ckbChain
  const validators = network_config.ethereum_private_keys
    .slice(0, 2)
    .map((privateKey) => {
      let publicKey = EthUtil.privateToPublic(Buffer.from(privateKey, 'hex'));
      return '0x' + EthUtil.publicToAddress(publicKey).toString('hex');
    });
  console.error('validator validator: ', validators);
  const multisigThreshold = 1;
  let eth_network = await provider.getNetwork();
  const chainId = eth_network.chainId;
  console.error('chain id :', chainId);

  let factory = await ethers.getContractFactory(
    'contracts/CKBChain.sol:CKBChain'
  );
  let CKBChain = await upgrades.deployProxy(
    factory,
    [validators, multisigThreshold],
    {
      initializer: 'initialize',
      unsafeAllowCustomTypes: true,
      unsafeAllowLinkedLibraries: true,
    }
  );
  const CKBChainAddr = CKBChain.address;
  console.error('CKBChain address: ', CKBChainAddr);
  const waitingSeconds = 20;
  console.error(`waiting ${waitingSeconds} seconds`);
  await sleep(waitingSeconds);

  // deploy TokenLocker
  const numConfirmations = 0;
  factory = await ethers.getContractFactory(
    'contracts/TokenLocker.sol:TokenLocker'
  );
  const locker = await upgrades.deployProxy(
    factory,
    [
      CKBChainAddr,
      numConfirmations,
      '0x' + recipient_typescript_code_hash,
      recipientCellTypescriptHashType,
      lightClientTypescriptHash,
      '0x' + bridge_lockscript_code_hash,
    ],
    {
      initializer: 'initialize',
      unsafeAllowCustomTypes: true,
      unsafeAllowLinkedLibraries: true,
    }
  );

  const lockerAddr = locker.address;
  console.error('tokenLocker address: ', lockerAddr);
  console.error(`waiting ${waitingSeconds} seconds`);
  await sleep(waitingSeconds);

  // write eth address to settings
  deployedContracts.eth_token_locker_addr = lockerAddr;
  deployedContracts.eth_ckb_chain_addr = CKBChainAddr;
  deployedContracts.ckb_relay_mutlisig_threshold.threshold = multisigThreshold;
  const new_config = TOML.stringify(forceConfig);
  fs.writeFileSync(forceConfigPath, new_config);
  console.error('write eth addr into config successfully');

  const tokenLockerJson = require('../artifacts/contracts/TokenLocker.sol/TokenLocker.json');
  const lockerABI = tokenLockerJson.abi;
  const ckbChainJSON = require('../artifacts/contracts/CKBChain.sol/CKBChain.json');
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
