const { log } = require("../../test/utils");

async function main() {
  // deploy hamstersong
  let factory = await ethers.getContractFactory(
    "contracts/Eaglesong.sol:Eaglesong"
  );
  const hamstersong = await factory.deploy();
  await hamstersong.deployTransaction.wait(1);
  const hamAddr = hamstersong.address;
  const provider = hamstersong.provider;
  log(`hamstersong address: ${hamAddr}`);

  // deploy TestEaglesong
  factory = await ethers.getContractFactory(
    "contracts/test/TestEaglesong.sol:TestHamstersong"
  );
  const testEaglesong = await factory.deploy(hamAddr);
  await testEaglesong.deployTransaction.wait(1);
  log(`testHam address: ${testEaglesong.address}`);

  // calc Eaglesong
  let res = await testEaglesong.callStatic.ckbEaglesong(
    "0xcbecbaf6a2deee59b2eab3bbae5388128ce9f30183336526c9081419f163fc6076030000312b000000000000216033d200000000000000000000000000000000"
  );
  log(res);

  // calc gas
  res = await testEaglesong.ckbEaglesong(
    "0xcbecbaf6a2deee59b2eab3bbae5388128ce9f30183336526c9081419f163fc6076030000312b000000000000216033d200000000000000000000000000000000"
  );
  const receipt = await res.wait(1);
  log(`gasUsed: ${receipt.gasUsed}`);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
