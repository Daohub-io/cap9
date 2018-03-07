pragma solidity ^0.4.17;

import "truffle/Assert.sol";
import "truffle/DeployedAddresses.sol";
import "../contracts/Factory.sol";
import "../contracts/test/valid/Adder.sol";


contract TestFactory {

    Adder a;
    function testShouldCreateAdder() public {
        Factory factory = new Factory();
        a = Adder(factory.createProcedure(hex"6060604052341561000f57600080fd5b60ba8061001d6000396000f300606060405260043610603f576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063771602f7146044575b600080fd5b3415604e57600080fd5b606b60048080359060200190919080359060200190919050506081565b6040518082815260200191505060405180910390f35b60008183019050929150505600a165627a7a72305820208e94342b2b01f28a6a3e363d85bb930b900adbc78c5ac5c926c3c316c993840029"));
        Assert.notEqual(a, 0, "Invalid Address");
        Assert.equal(2, a.add(1,1), "Invalid Contract");
    }
}
