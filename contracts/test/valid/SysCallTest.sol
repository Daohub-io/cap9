pragma solidity ^0.4.17;

contract SysCallTest {
    function S() public {
        assembly {
            // First get the original value from storage
            let orig_value := sload(0x8000)
            // First set up the input data (at memory location 0x0)
            // The write call is 0x-00-00-02
            mstore(0x0,0x000002)
            // The storage location we want is 0x8000
            mstore(0x20,0x8000)
            // The value we want to store
            mstore(0x40,add(orig_value,1))
            // "in_offset" is at 29, because we only want the last 3 bytes
            // "in_size" is 67 because it is 3+32+32
            // we will store the result at 0x80 and it will be 1 byte
            delegatecall(gas, caller, 29, 67, 0x80, 1)
            mstore(0xd,sload(0x8000))
            if iszero(eq(sload(0x8000),add(orig_value,1))) {
                mstore(0xd,999)
                revert(0xd,0x20)
            }
            log0(0xd,32)
            // Store the result of delegatecall at 0x60
            0x60
            mstore
            // return both the delegatecall return value and the system call
            // retun value
            return(0xd,0x20)
        }
    }
}