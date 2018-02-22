var Kernel = artifacts.require("./Kernel.sol");

module.exports = function(deployer) {
  deployer.deploy(Kernel);
};
