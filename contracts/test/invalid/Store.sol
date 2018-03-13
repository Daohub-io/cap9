pragma solidity ^0.4.17;

contract Store {

    function foo() {
        uint256 foo = 1234;
        assembly {
            sstore(0x0, mload(foo))
        }
    }
}