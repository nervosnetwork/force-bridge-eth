const { expect } = require("chai");

contract("TokenLocker", () => {
    let tokenLocker;

    before(async () => {
        let factory = await ethers.getContractFactory("MockCKBSpv")
        const mockSpv = await factory.deploy()
        await mockSpv.deployed()

        factory = await ethers.getContractFactory("TokenLocker")
        tokenLocker = await factory.deploy(mockSpv.address, 123)
        await tokenLocker.deployed()
    });

    describe("lockETH", async function () {
        it("Should ", async function () {
            // let defaultProvider = ethers.getDefaultProvider();
            const defaultProvider = tokenLocker.signer.provider
            let contractBalance = await defaultProvider.getBalance(tokenLocker.address)

            // lockETH
            const amount = ethers.utils.parseEther("1.2")
            await tokenLocker.lockETH([0], 0, {value: amount});

            // asset expected amount == balance of contract delta
            const delta = await defaultProvider.getBalance(tokenLocker.address) - contractBalance
            const actualDelta = ethers.BigNumber.from(delta.toString())
            expect(actualDelta).to.equal(amount)
        });
    });

    describe("unlockETH", async function () {
        it("Should ", async function () {
            // let defaultProvider = ethers.getDefaultProvider();
            const defaultProvider = tokenLocker.signer.provider
            let contractBalance = await defaultProvider.getBalance(tokenLocker.address)

            // unlockETH
            await tokenLocker.unlockToken([0]);

            // asset expected amount == balance of contract delta
            const delta = await defaultProvider.getBalance(tokenLocker.address) - contractBalance
            const actualDelta = ethers.BigNumber.from(delta.toString())
            const expected = ethers.BigNumber.from("-111100000000000000")
            expect(actualDelta).to.equal(expected)
        });
    });

})
