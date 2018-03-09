pragma solidity ^0.4.17;

contract Callcode {
    function foo(address a, uint gasAmount) public returns (bool v) {
        assembly {
            v := callcode(gasAmount, a, 0, 0, 0, 0, 0)
        }
    }
}