pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestEntry is BeakerContract {
    // Log to no topics
    function A() public returns (uint32) {
        return set_entry(0, "TestWrite");
    }
}
