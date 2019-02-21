pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestCaps is BeakerContract {
    // Push a capability to a procedure.
    function PushCap(bytes24 procedureName, uint256[] cap) public returns (uint32) {
        log0("PushCap");
        // return 7;
        return cap_push(0, bytes32(procedureName), cap);
    }

    function testNum() public pure returns (uint256) {
        return 567;
    }
}
