var Migrations = artifacts.require("./Migrations.sol");
var Factory = artifacts.require("./Factory.sol");

module.exports = function(deployer) {
  deployer.deploy(Migrations);
  deployer.deploy(Factory);
};
