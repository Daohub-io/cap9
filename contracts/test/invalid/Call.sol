pragma solidity ^0.4.17;

contract Call {
    function foo(address a, uint gasAmount) public returns (bool v) {
        assembly {
            v := call(gasAmount, a, 0, 0, 0, 0,0)
        }
    }
}