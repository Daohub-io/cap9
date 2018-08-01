pragma solidity ^0.4.17;

contract Delegatecall {
    function foo(address a, uint gasAmount) public returns (bool v) {
        assembly {
            v := delegatecall(gasAmount, a, 0, 0, 0, 0)
        }
    }
}