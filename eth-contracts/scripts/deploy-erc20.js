const { deployAll } = require('../test/utils');

async function main() {
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

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
