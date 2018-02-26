pragma solidity ^0.4.17;

contract Delegatecall {
    function foo(address a, uint gas) public returns (bool v) {
        assembly {
            v := delegatecall(gas, a, 0, 0, 0, 0)
        } 
    }
}