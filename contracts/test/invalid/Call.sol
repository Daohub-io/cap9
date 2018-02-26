pragma solidity ^0.4.17;

contract Call {
    function foo(address a, uint gas) public returns (bool v) {
        assembly {
            v := call(gas, a, 0, 0, 0, 0,0)
        }
    } 
}