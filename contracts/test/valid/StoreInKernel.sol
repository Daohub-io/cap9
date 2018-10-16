pragma solidity ^0.4.17;

contract StoreInKernel {

    function foo() public {
        uint256 fooVal = 1234;
        assembly {
            mload(fooVal)
            0x0

            // This code is necessary in front of an SSTORE to pass verification
            0x64
            mload(0x40)
            mstore

            sstore
        }
    }
}