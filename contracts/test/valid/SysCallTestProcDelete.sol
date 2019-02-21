pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestProcDelete is BeakerContract {
    // Register a procedure
    function Delete(bytes24 name) public returns (uint32) {
        return proc_del(0, bytes32(name));
        // return name;
    }

    function testNum() public pure returns (uint256) {
        return 982;
    }
}
