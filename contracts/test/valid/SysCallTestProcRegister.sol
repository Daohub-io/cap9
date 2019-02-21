pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestProcRegister is BeakerContract {
    // Register a procedure
    function A(bytes24 name, address procAddress) public returns (uint32) {
        return proc_reg(1, bytes32(name), procAddress);
    }

    function testNum() public pure returns (uint256) {
        return 392;
    }
}
