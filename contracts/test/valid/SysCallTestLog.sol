pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestLog is BeakerContract {
    // Log to no topics
    function A() public returns (uint32) {
        return log0(1, uint32(0x1234567890));
    }

    // Log to a single topic
    function B() public {
        log1(1, 0xabcd, uint32(0x1234567890));
    }

    // Log to two topics
    function C() public {
        log2(1, 0xabcd, 0xbeef, uint32(0x1234567890));
    }

    // Log to three topics
    function D() public {
        log3(1, 0xabcd, 0xbeef, 0xcafe, uint32(0x1234567890));
    }

    // Log to four topics
    function E() public {
        log4(1, 0xabcd, 0xbeef, 0xcafe, 0x4545, uint32(0x1234567890));

    }
}