pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestCaps is BeakerContract {
    // Push a capability to a procedure.
    function PushCap(bytes24 procedureName, uint256[] cap) public returns (uint32) {
        log0("PushCap");
        // return 7;
        return cap_push(0, bytes32(procedureName), cap);
    }

    // Register a procedure with capabilities
    function B(bytes24 name, address procAddress, uint256[] caps) public returns (uint32) {
        return proc_reg(1, bytes32(name), procAddress, caps);
    }

    function testNum() public pure returns (uint256) {
        return 567;
    }
}
