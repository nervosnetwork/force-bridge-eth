const TokenLocker = artifacts.require("TokenLocker");
const spv = artifacts.require("MockCKBSpv");

module.exports = function (deployer) {
  deployer.deploy(spv).then(function() {
    return deployer.deploy(TokenLocker, spv.address, 20);
  });
};
