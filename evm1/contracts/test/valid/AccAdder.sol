pragma solidity ^0.4.17;

contract AccAdder {
    function add(uint256 a, uint256 b) public payable returns (uint256) {
        return a + b;
    }
}
