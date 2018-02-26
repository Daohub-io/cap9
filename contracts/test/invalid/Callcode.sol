pragma solidity ^0.4.17;

contract Callcode {
    function foo(address a, uint gas) public returns (bool v) {
        assembly {
            v := callcode(gas, a, 0, 0, 0, 0, 0)
        }
    }
}