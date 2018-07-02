pragma solidity ^0.4.17;

contract StoreInTable1 {
    function foo() {
        uint256 foo = 1234;
        assembly {
            mload(foo)
            0x0100000000000000000000000000000000000000000000000000000000000000
            // This code is necessary in front of an SSTORE to pass verification
            0x64
            mload(0x40)
            mstore
            sstore
        }
    }
    function boo() {
        uint256 loo = 1234;
        assembly {
            mload(loo)
            0x0100000100000000000000000000000000000000000000000000000000000000
            // This code is necessary in front of an SSTORE to pass verification
            0x64
            mload(0x40)
            mstore
            sstore
        }
    }
}
