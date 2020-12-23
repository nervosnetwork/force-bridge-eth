require("@nomiclabs/hardhat-truffle5");
require("@nomiclabs/hardhat-waffle");
require("hardhat-gas-reporter");
require("solidity-coverage");
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
        compilers: [
            {
                version: "0.5.10",
                settings: {
                    optimizer: {
                        enabled: true,
                        runs: 200
                    }
                }
            },
            {
                version: "0.6.6",
                settings: {
                    optimizer: {
                        enabled: true,
                        runs: 200
                    }
                }
            },
            {
                version: "0.7.4",
                settings: {
                    optimizer: {
                        enabled: false,
                        runs: 200
                    }
                }
            },
            {
                version: "0.8.0",
                settings: {
                    optimizer: {
                        enabled: false,
                        runs: 200
                    }
                }
            }
        ]
    },

    defaultNetwork: "hardhat",

    networks: {
        hardhat: {},
        ropsten: {
            url: `${process.env.ROPSTEN_API ? process.env.ROPSTEN_API : "https://ropsten.infura.io/v3/48be8feb3f9c46c397ceae02a0dbc7ae"}`,
            accounts: [`${process.env.ROPSTEN_DEPLOYER_PRIVATE_KEY ? process.env.ROPSTEN_DEPLOYER_PRIVATE_KEY : "0x49740e7b29259e7c2b693f365a9fd581cef75d1e346c8dff89ec037cdfd9f89d"}`]
        },
        kovan: {
            url: `${process.env.KOVAN_API ? process.env.KOVAN_API : "https://kovan.infura.io/v3/3ed3eadf912c4b31b800aafeedbf79eb"}`,
            accounts: [`${process.env.KOVAN_DEPLOYER_PRIVATE_KEY ? process.env.KOVAN_DEPLOYER_PRIVATE_KEY : "0xc4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a"}`]
        },
        geth: {
            url: `http://127.0.0.1:8545`,
            // address [`0x17c4b5CE0605F63732bfd175feCe7aC6b4620FD2`, `0x46beaC96B726a51C5703f99eC787ce12793Dae11`]
            // Mnemonic [`dignity vehicle fuel siren cool machine video spice oppose olympic polar discover`, ``]
            accounts: [`0xc4ad657963930fbff2e9de3404b30a4e21432c89952ed430b56bf802945ed37a`, `0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc`],
        },
        ganache: {
            url: `http://127.0.0.1:7545`,
            // address [`0x10A4c9D160196086Ab8D4247e2D607f910e8cac0`]
            // Mnemonic [`live blush purchase cheap obey spare purchase depend remain truly slush assist`]
            accounts: [`0x719e94ec5d2ecef67b5878503ffd6e1e0e2fe7a52ddd55c436878cb4d52d376d`],
        }
    },

    paths: {
        sources: "./contracts",
        tests: "./test",
        cache: "./cache",
        artifacts: "./artifacts"
    }
};
