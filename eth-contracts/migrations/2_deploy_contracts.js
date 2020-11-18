const TokenLocker = artifacts.require("TokenLocker");
const MockCKBSpv = artifacts.require("MockCKBSpv");
const ERC20 = artifacts.require("test/ERC20");

module.exports = function (deployer) {
    deployer.deploy(ERC20);
    deployer.deploy(MockCKBSpv).then(function() {
        return deployer.deploy(TokenLocker, MockCKBSpv.address, 20,"0x2a1acd26847576a128e3dba3aa984feafffdf81f7c7b23bdf51e7bec1c15944c", 1);
    });

};
