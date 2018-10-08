pragma solidity ^0.4.17;

contract SysCallTest {
    function S() public {
        log0(bytes32("performing S()"));
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
            // clear the output buffer
            mstore(0x80,0)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,sload(0x8000))
            if iszero(eq(sload(0x8000),add(orig_value,1))) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            return(0x80,0x20)
        }
    }

    function() public {
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
            mstore(0x60,356)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, 0x20)) {
                mstore(0xd,add(2500,mload(0x80)))
                revert(0xd,0x20)
            }
            return(0x80, 0x20)
        }
    }
}