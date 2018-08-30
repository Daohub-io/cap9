pragma solidity ^0.4.17;

contract SysCallTest {
    function S() public {
        assembly {
            // First get the original value from storage
            let orig_value := sload(0x8000)
            // First set up the input data (at memory location 0x0)
            // The write call is 0x-07
            mstore(0x0,0x07)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The storage location we want is 0x8000
            mstore(0x40,0x8000)
            // The value we want to store
            mstore(0x60,add(orig_value,1))
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, 0x20)) {
                // If delegate call reverted with a value of 11, the action
                // was rejected as the capability was incorrect
                if eq(11,mload(0x80)) {
                    mstore(0xd,905)
                    revert(0xd,0x20)
                }
                if eq(12,mload(0x80)) {
                    mstore(0xd,906)
                    revert(0xd,0x20)
                }
                if eq(888,mload(0x80)) {
                    mstore(0xd,888)
                    revert(0xd,0x20)
                }
                if eq(889,mload(0x80)) {
                    mstore(0xd,889)
                    revert(0xd,0x20)
                }
                // If delegate call returns an error code, revert with 909
                mstore(0xd,add(1000,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,sload(0x8000))
            if iszero(eq(sload(0x8000),add(orig_value,1))) {
                mstore(0xd,999)
                revert(0xd,0x20)
            }
            log0(0xd,32)
            // return both the delegatecall return value and the system call
            // retun value
            return(0xd,0x20)
        }
    }
}