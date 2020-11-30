const BN = require("bn.js");
const { expect } = require("chai");
const { sleep, log } = require("../../test/utils");

contract("Faucet", () => {
  const contracts = [];

  before(async function () {
    // disable timeout
    this.timeout(0);

    // deploy all erc20 tokens
    const factoryPaths = [
      "contracts/test/ERC20.sol:DAI",
      "contracts/test/ERC20.sol:USDT",
      "contracts/test/ERC20.sol:USDC",
    ];
    const promises = [];
    for (const path of factoryPaths) {
      const factory = await ethers.getContractFactory(path);
      const contract = await factory.deploy();
      contracts.push(contract);
      promises.push(contract.deployTransaction.wait(1));
      await sleep(1);
    }
    await Promise.all(promises);
    log(`
  DAIAddr: ${contracts[0].address}, USDTAddr: ${contracts[1].address}, USDCAddr: ${contracts[2].address}
  `);
  });

  describe("DAI Faucet correct case", async function () {
    // disable timeout
    this.timeout(0);

    it("Should getTestToken verified", async () => {
      let res, receipt;
      const testUser = "0x1234567890123456789012345678901234567890";
      for (const erc20 of contracts) {
        const decimal = await erc20.callStatic.decimals();
        const supply = 100 * Math.pow(10, decimal);
        const beforeBalance = await erc20.callStatic.balanceOf(testUser);
        res = await erc20.getTestToken(testUser);
        receipt = await res.wait(1);
        const afterBalance = await erc20.callStatic.balanceOf(testUser);
        expect(afterBalance - beforeBalance).to.equal(supply);
      }
    });

    it("Should mint amount verified", async () => {
      let res, receipt;
      for (const erc20 of contracts) {
        const signerAddr = erc20.signer.address;
        const beforeBalance = await erc20.callStatic.balanceOf(signerAddr);
        const amount = ethers.utils.parseEther("123");
        res = await erc20.mint(amount);
        receipt = await res.wait(1);
        const afterBalance = await erc20.callStatic.balanceOf(signerAddr);
        expect((afterBalance - beforeBalance).toString()).to.equal(
          amount.toString()
        );
      }
    });
  });
});
