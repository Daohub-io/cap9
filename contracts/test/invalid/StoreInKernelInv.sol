pragma solidity ^0.4.17;

contract StoreInKernelInv {

    function foo() public {
        uint256 fooVal = 1234;
        assembly {
            mload(fooVal)
            0x0
            // This lacks the necessary protection code
            sstore
        }
    }
}