const { expect } = require("chai");
const { log, sleep, deployContract } = require('./utils')

const deployAll = async (contractPaths) => {
  const contracts = []
  const promises = []
  for (const path of contractPaths) {
    const factory = await ethers.getContractFactory(path);
    const contract = await factory.deploy();
    contracts.push(contract);
    promises.push(contract.deployTransaction.wait(1));
    // because nonce should increase in sequence
    await sleep(1);
  }

  await Promise.all(promises)
  return contracts
}

contract("Proxy", () => {
  let storageContract, storageContractAddress;
  let contractPaths
  let v1Address, v1Instance
  let v2Address, v2Instance
  let v3Address, v3Instance
  let adminAddress;
  const slot1Amount = 111

  before(async function () {
    // disable timeout
    this.timeout(0);

    const [signer] = await ethers.getSigners();
    adminAddress = signer.address

    // deploy storageContract, storageContractAddress is the only contract address for outside users
    storageContract = await deployContract(
      "contracts/test/proxy/ProxyTestStorage.sol:ProxyTestStorage",
        1,
        adminAddress
    );
    storageContractAddress = storageContract.address;

    // 3 versions of the same contract, v1 -> v2 -> v3
    // upgrade the same contract, from v1 to v2, from v2 to v3, using the only one contract address storageContractAddress
    // storageContractAddress is the only contract address for these 3 contracts
    contractPaths = [
      "contracts/test/proxy/v1/ProxyTestLogicV1.sol:ProxyTestLogicV1",
      "contracts/test/proxy/v2/ProxyTestLogicV2.sol:ProxyTestLogicV2",
      "contracts/test/proxy/v3/ProxyTestLogicV3.sol:ProxyTestLogicV3",
    ]
  });

  describe("deploy v1 contract and use v1 contract", async function () {
    // disable timeout
    this.timeout(0);
    it("v1 contract test", async () => {
      // deploy v1 contract
      const v1Contract = await deployContract(contractPaths[0])
      v1Address = v1Contract.address
      v1Instance = await ethers.getContractAt(contractPaths[0], storageContractAddress)
      log(`v1Contract: `, v1Address)

      // register v1Contract to storageContract Routers
      let txRes = await storageContract.sysAddDelegates([v1Address])
      await txRes.wait(1)
      expect(await storageContract.callStatic.sysCountDelegate()).to.equal(1)

      // send v1Contract.setSlot1(uint256)
      log(`v1Instance set slot1 =`, slot1Amount)
      txRes = await v1Instance.setSlot1(slot1Amount)
      await txRes.wait(1)
      expect(await v1Instance.callStatic.getSlot1()).to.equal(slot1Amount)

      // send v1Contract.fnSlot1(): slot1++
      log(`v1Instance fnSlot1()`, slot1Amount)
      txRes = await v1Instance.fnSlot1()
      await txRes.wait(1)
      expect(await v1Instance.callStatic.getSlot1()).to.equal(slot1Amount + 1)
    });

    it("upgrade v1Contract to v2Contract", async () => {
      // deploy v2
      const v2Contract = await deployContract(contractPaths[1])
      v2Address = v2Contract.address
      v2Instance = await ethers.getContractAt(contractPaths[1], storageContractAddress)
      log(`v2Contract: `, v2Address)

      // register v2Contract to storageContract Routers
      let txRes = await storageContract.sysAddDelegates([v2Address])
      await txRes.wait(1)
      expect(await storageContract.callStatic.sysCountDelegate()).to.equal(2)

      // send v2Contract.setSlot3(bytes32)
      log(`v2Instance test`)
      log(await v2Instance.callStatic.getSlot3())

      const slot3Amount = '0x0000000000000000000000000000000000000000000000000000000000000123'
      txRes = await v2Instance.setSlot3(slot3Amount)
      await txRes.wait(1)
      expect(await v2Instance.callStatic.getSlot3()).to.equal(slot3Amount)
    });

    it("upgrade v2Contract to v3Contract", async () => {
      // deploy v3
      const v3Contract = await deployContract(contractPaths[2])
      v3Address = v3Contract.address
      v3Instance = await ethers.getContractAt(contractPaths[2], storageContractAddress)
      log(`v3Contract: `, v3Address)

      // register v3Contract to storageContract Routers
      let txRes = await storageContract.sysAddDelegates([v3Address])
      await txRes.wait(1)
      expect(await storageContract.callStatic.sysCountDelegate()).to.equal(3)
      
      log(`delete v1Address from storageContract Routers`)
      txRes = await storageContract.sysDelDelegates([v1Address])
      await txRes.wait(1)
      expect(await storageContract.callStatic.sysCountDelegate()).to.equal(2)

      // send v2Contract.setSlot3(bytes32)
      log(`v2Instance calls`)
      log(await v2Instance.callStatic.getSlot3())
      const slot3Amount = '0x0000000000000000000000000000000000000000000000000000000000000456'
      txRes = await v2Instance.setSlot3(slot3Amount)
      await txRes.wait(1)
      expect(await v2Instance.callStatic.getSlot3()).to.equal(slot3Amount)

      // send v3Contract.setSlot5(uint256, uint256)
      log(`v3Instance calls`)
      log(await v3Instance.callStatic.getSlot5())
      txRes = await v3Instance.setSlot5(5, 6)
      await txRes.wait(1)
      const [left, right] = await v3Instance.callStatic.getSlot5()
      expect(left.toNumber()).to.equal(5)
      expect(right.toNumber()).to.equal( 6)

      // send v3Contract.fnSlot1(): slot1 = slot1 * 2
      log(`v3Instance fnSlot1`)
      txRes = await v3Instance.fnSlot1()
      await txRes.wait(1)
      expect(await v3Instance.callStatic.getSlot1()).to.equal((slot1Amount + 1) * 2)

      // because v3Instance lacks of `setSlot1` abi function
      try {
        await v3Instance.setSlot1(222)
      } catch (e) {
        log(e.toString())
      }

      // because `v1Instance` has been deleted from storageContract Routers
      try {
        await v1Instance.setSlot1(222)
      } catch (e) {
        log(e.toString())
      }
    });
  });
});
