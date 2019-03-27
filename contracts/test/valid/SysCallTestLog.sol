pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestLog is BeakerContract {
    // Log to no topics
    function A() public returns (uint32) {
        return proc_log0(0, uint32(0x1234567890));
    }

    // Log to a single topic
    function B() public {
        proc_log1(0, 0xabcd, uint32(0x1234567890));
    }

    // Log to two topics
    function C() public {
        proc_log2(0, 0xabcd, 0xbeef, uint32(0x1234567890));
    }

    // Log to three topics
    function D() public {
        proc_log3(0, 0xabcd, 0xbeef, 0xcafe, uint32(0x1234567890));
    }

    // Log to four topics
    function E() public {
        proc_log4(0, 0xabcd, 0xbeef, 0xcafe, 0x4545, uint32(0x1234567890));

    }
}
