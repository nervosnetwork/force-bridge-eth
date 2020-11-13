// We require the Buidler Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
// When running the script with `buidler run <script>` you'll find the Buidler
// Runtime Environment's members available in the global scope.
const { waitingForReceipt } = require("../../test/utils");
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
    "contracts/test/TestEaglesongLib.sol:TestEaglesongLib"
  );
  const contract = await factory.deploy();
  await contract.deployed();
  const contractAddr = contract.address;
  console.log("TestEaglesong deployed to:", contractAddr);

  // calc Eaglesong
  let res = await contract.callStatic.ckbEaglesong(
    "0xcbecbaf6a2deee59b2eab3bbae5388128ce9f30183336526c9081419f163fc6076030000312b000000000000216033d2"
  );

  assert(
    res ===
      "0x000000000000053ee598839a89638a5b37a7cf98ecf0ce6d02d3d9287f008b84",
    `${res} !== 0x000000000000053ee598839a89638a5b37a7cf98ecf0ce6d02d3d9287f008b84`
  );

  // calc Eaglesong for hamster
  res = await contract.callStatic.ckbEaglesong(
    "0x6162636465666768696a6b6c6d6e6f707172737475767778797a303132333435363738396162636465666768696a6b6c"
  );

  console.log(`hamster res = ${res}`);

  const txReceipt = await waitingForReceipt(contract.provider, res);
  console.log("txReceipt: ", txReceipt);
  console.log("gasUsed: ", txReceipt.gasUsed.toString());
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
