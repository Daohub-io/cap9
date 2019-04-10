pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestProcRegister is BeakerContract {
    // Register a procedure
    function A(bytes24 name, address procAddress) public returns (uint32) {
        return proc_reg(0, bytes32(name), procAddress, new uint256[](0));
    }

    // Register a procedure with capabilities
    function B(bytes24 name, address procAddress, uint256[] caps) public returns (uint32) {
        return proc_reg(0, bytes32(name), procAddress, caps);
    }

    // Delete a procedure
    function Delete(bytes24 name) public returns (uint32) {
        return proc_del(0, bytes32(name));
    }


    function testNum() public pure returns (uint256) {
        return 392;
    }
}
