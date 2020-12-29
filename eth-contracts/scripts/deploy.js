const fs = require('fs');
const TOML = require('@iarna/toml');
const EthUtil = require('ethereumjs-util');
const {
  deployUpgradableContractFirstTime,
  ckbBlake2b,
} = require('../test/utils');

async function main() {
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

  const wallet = new ethers.Wallet(
    '0x' + network_config.ethereum_private_keys[0],
    provider
  );
  const adminAddress = wallet.address;

  // deploy ckbChainV2
  const validators = network_config.ethereum_private_keys.map((privateKey) => {
    let publicKey = EthUtil.privateToPublic(Buffer.from(privateKey, 'hex'));
    return '0x' + EthUtil.publicToAddress(publicKey).toString('hex');
  });
  console.error('validator validator: ', validators);
  const multisigThreshold = 5;
  let eth_network = await provider.getNetwork();
  const chainId = eth_network.chainId;
  console.error('chain id :', chainId);

  // deploy CKBChainV2
  const canonicalGcThreshold = 40000;
  let CKBChainV2 = await deployUpgradableContractFirstTime(
    'contracts/CKBChainV2Storage.sol:CKBChainV2Storage',
    'contracts/CKBChainV2Logic.sol:CKBChainV2Logic',
    adminAddress,
    canonicalGcThreshold,
    validators,
    multisigThreshold
  );
  const ckbChainV2Addr = CKBChainV2.address;

  // deploy TokenLocker
  const numConfirmations = 10;
  const locker = await deployUpgradableContractFirstTime(
    'contracts/TokenLockerStorage.sol:TokenLockerStorage',
    'contracts/TokenLockerLogic.sol:TokenLockerLogic',
    adminAddress,
    ckbChainV2Addr,
    numConfirmations,
    '0x' + recipient_typescript_code_hash,
    recipientCellTypescriptHashType,
    lightClientTypescriptHash,
    '0x' + bridge_lockscript_code_hash
  );

  const lockerAddr = locker.address;
  console.error('tokenLocker address: ', lockerAddr);

  // write eth address to settings
  deployedContracts.eth_token_locker_addr = lockerAddr;
  deployedContracts.eth_ckb_chain_addr = ckbChainV2Addr;
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
