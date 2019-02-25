pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestCaps is BeakerContract {
    // Push a capability to a procedure.
    function PushCap(bytes24 procedureName, uint256[] cap) public returns (uint32) {
        log0("PushCap");
        return cap_push(0, bytes32(procedureName), cap);
    }

    // Delte a capability from a procedure.
    function DeleteCap(bytes24 procedureName, uint256 capIndex) public returns (uint32) {
        log0("DeleteCap");
        return cap_del(1, bytes32(procedureName), capIndex);
    }

    function testNum() public pure returns (uint256) {
        return 567;
    }
}
