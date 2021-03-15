// We require the Buidler Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
// When running the script with `buidler run <script>` you'll find the Buidler
// Runtime Environment's members available in the global scope.
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  // Buidler always runs the compile task when running scripts through it.
  // If this runs in a standalone fashion you may want to call compile manually
  // to make sure everything is compiled
  // await bre.run('compile');

  // deploy TestBlake2b
  const factory = await ethers.getContractFactory(
    'contracts/test/TestBlake2b.sol:TestBlake2b'
  );
  const contract = await factory.deploy();
  await contract.deployed();
  const contractAddr = contract.address;
  console.log('TestBlake2b deployed to:', contractAddr);

  // calc blake2b
  console.log('waiting for block confirmations, about 10s');
  await sleep(10 * 1000);
  let res = await contract.ckbBlake2b(
    '0x39e33c8ad2e7e4eb71610d2bcdfbb0cb0fde2f96418256914ad2f5be1d6e9331',
    '0x385dfb0153a0e3aec760120c4e333a4a6bec91eeaca359ef714709588d23ca16'
  );
  assert(
    res ===
      '0x93a9faceb827e8a431217f0e5fc6068c14cc62ac4cf73752ed4e9135adc364c8',
    `${res} !== 0x93a9faceb827e8a431217f0e5fc6068c14cc62ac4cf73752ed4e9135adc364c8`
  );
  console.log('TestBlake2b success');
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
