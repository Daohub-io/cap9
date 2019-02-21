pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestSetEntry is BeakerContract {
    // Register a procedure
    function SetEntry(bytes24 name) public returns (uint32) {
        return set_entry(0, bytes32(name));
        // return name;
    }

    function testNum() public pure returns (uint256) {
        return 773;
    }
}
