var Migrations = artifacts.require("./Migrations.sol");
var Factory = artifacts.require("./Factory.sol");
var Kernel = artifacts.require("./Kernel.sol");

module.exports = function(deployer) {
  deployer.deploy(Migrations);
  deployer.deploy(Factory);
  deployer.deploy(Kernel);
};
