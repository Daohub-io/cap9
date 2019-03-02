pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestProcDelete is BeakerContract {
    // Register a procedure
    function Delete(bytes24 name) public returns (uint32) {
        return proc_del(0, bytes32(name));
        // return name;
    }

    // Register a procedure with capabilities
    function B(bytes24 name, address procAddress, uint256[] caps) public returns (uint32) {
        return proc_reg(1, bytes32(name), procAddress, caps);
    }

    function testNum() public pure returns (uint256) {
        return 982;
    }
}
