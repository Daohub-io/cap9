pragma solidity ^0.4.17;

contract StoreInKernelInv {

    function foo() {
        uint256 foo = 1234;
        assembly {
            mload(foo)
            0x0
            // This lacks the necessary protection code
            sstore
        }
    }
}