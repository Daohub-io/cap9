var Migrations = artifacts.require("./Migrations.sol");
var Kernel = artifacts.require("./Kernel.sol");

module.exports = function(deployer) {
  deployer.deploy(Migrations);
  deployer.deploy(Kernel);
};
