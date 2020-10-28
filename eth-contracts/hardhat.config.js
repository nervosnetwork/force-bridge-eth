require("@nomiclabs/hardhat-truffle5");
require("@nomiclabs/hardhat-waffle");

require('dotenv').config()

// This is a sample Buidler task. To learn how to create your own go to
// https://buidler.dev/guides/create-task.html
task("accounts", "Prints the list of accounts", async () => {
    const accounts = await ethers.getSigners();

    for (const account of accounts) {
        console.log(await account.getAddress());
    }
});

// You have to export an object to set up your config
// This object can have the following optional entries:
// defaultNetwork, networks, solc, and paths.
// Go to https://buidler.dev/config/ to learn more
module.exports = {
    // This is a sample solc configuration that specifies which version of solc to use
    solidity: {
        version: "0.5.10",
    },

    defaultNetwork: "hardhat",

    networks: {
        hardhat: {},
        ropsten: {
            url: `${process.env ? process.env.ROPSTEN_API : ""}`,
            accounts: [`${process.env ? process.env.ROPSTEN_DEPLOYER_PRIVATE_KEY : ""}`]
        },
        kovan: {
            url: `${process.env ? process.env.KOVAN_API : ""}`,
            accounts: [`${process.env ? process.env.KOVAN_DEPLOYER_PRIVATE_KEY : ""}`]
        },
        geth: {
            url: `http://127.0.0.1:8543`,
            // address [`0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2`]
            // Mnemonic [`dignity vehicle fuel siren cool machine video spice oppose olympic polar discover`]
            accounts: [`0xc4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a`],
        }
    },

    paths: {
        sources: "./contracts",
        tests: "./test",
        cache: "./cache",
        artifacts: "./artifacts"
    }
};
