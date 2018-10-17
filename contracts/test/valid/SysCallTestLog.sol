pragma solidity ^0.4.17;

contract SysCallTestLog {
    // Log to no topics
    function A() public {
        assembly {
            function mallocZero(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // zero-out the memory
                // if there are some bytes to be allocated (rsize is not zero)
                if rsize {
                    // loop through the address and zero them
                    for { let n := 0 } iszero(eq(n, rsize)) { n := add(n, 32) } {
                        mstore(add(result,n),0)
                    }

                }
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            let ins := mallocZero(mul(4,32))
            let retSize := 0x20
            let retLoc := mallocZero(retSize)
            // First set up the input data (at memory location ins)
            // The log call is 0x-07
            mstore(add(ins,0x0),0x09)
            // The capability index is 0x-01
            mstore(add(ins,0x20),0x01)
            // The number of topics we will use
            mstore(add(ins,0x40),0x0)
            // The value we want to log
            mstore(add(ins,0x60),0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 97, retLoc, retSize)) {
                mstore(0xd,add(2200,mload(retLoc)))
                revert(0xd,0x20)
            }
            // We don't need to return anything in success
            return(retLoc,0)
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
            // We don't need to return anything in success
            return(0,0)
        }
    }

    // Log to two topics
    function C() public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(0x0,0x09)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The number of topics we will use
            mstore(0x40,0x2)
            // The first topic
            mstore(0x60,0xabcd)
            // The second topic
            mstore(0x80,0xbeef)
            // The value we want to log
            mstore(0xa0,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 129 because it is 1+32+32+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 161, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // We don't need to return anything in success
            return(0,0)
        }
    }

    // Log to three topics
    function D() public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(0x0,0x09)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The number of topics we will use
            mstore(0x40,0x3)
            // The first topic
            mstore(0x60,0xabcd)
            // The second topic
            mstore(0x80,0xbeef)
            // The third topic
            mstore(0xa0,0xcafe)
            // The value we want to log
            mstore(0xc0,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 129 because it is 1+32+32+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 193, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // We don't need to return anything in success
            return(0,0)
        }
    }

    // Log to four topics
    function E() public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(0x0,0x09)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The number of topics we will use
            mstore(0x40,0x4)
            // The first topic
            mstore(0x60,0xabcd)
            // The second topic
            mstore(0x80,0xbeef)
            // The third topic
            mstore(0xa0,0xcafe)
            // The fourth topic
            mstore(0xc0,0x4545)
            // The value we want to log
            mstore(0xe0,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 129 because it is 1+32+32+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 225, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // We don't need to return anything in success
            return(0,0)
        }
    }
}