pragma solidity ^0.4.17;

contract SysCallTestLog {
    // Log to no topics
    function A() public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The log call is 0x-07
            mstore(0x0,0x09)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The number of topics we will use
            mstore(0x40,0x0)
            // The value we want to log
            mstore(0x60,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,add(1100,mload(0x80)))
            return(0xd,0x20)
        }
    }

    // Log to a single topic
    function B() public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(0x0,0x09)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The number of topics we will use
            mstore(0x40,0x1)
            // The first topic
            mstore(0x60,0xabcd)
            // The value we want to log
            mstore(0x80,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 129 because it is 1+32+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 129, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // return both the delegatecall return value and the system call
            // retun value
            mstore(0xd,add(1100,mload(0x80)))
            return(0xd,0x20)
        }
    }
}