const TokenLocker = artifacts.require("TokenLocker");
const MockCKBSpv = artifacts.require("MockCKBSpv");

module.exports = function (deployer) {
    deployer.deploy(MockCKBSpv).then(function() {
        return deployer.deploy(TokenLocker, MockCKBSpv.address, 20);
    });
};
