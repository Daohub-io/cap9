pragma solidity ^0.4.17;

contract Adder {
    function add(uint a, uint b) public returns (uint256) {
        return a + b;
    }
    function test(uint a, uint b) public {
        assembly {
            mstore(0, add(a, b))
        }
    }
}