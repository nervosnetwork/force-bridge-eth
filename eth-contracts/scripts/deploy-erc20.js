const fs = require('fs');
const TOML = require('@iarna/toml');
const { deployAll } = require('../test/utils');

async function main() {
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
  const wallet = new ethers.Wallet(
    '0x' + network_config.ethereum_private_keys[0],
    provider
  );

  const contractPaths = [
    'contracts/test/ERC20.sol:DAI',
    'contracts/test/ERC20.sol:USDT',
    'contracts/test/ERC20.sol:USDC',
  ];
  const contracts = await deployAll(contractPaths);
  const [DAIAddr, USDTAddr, USDCAddr] = contracts.map(
    (contract) => contract.address
  );

  console.error(
    `DAIAddr: ${DAIAddr}, USDTAddr: ${USDTAddr}, USDCAddr: ${USDCAddr}`
  );
  const address = {
    daiContractAddr: DAIAddr,
    usdtContractAddr: USDTAddr,
    usdcContractAddr: USDCAddr,
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
