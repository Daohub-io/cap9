pragma solidity ^0.4.17;

contract Setter {
    function store(uint256 key, uint256 value) public {
        assembly {
            key
            value
            sstore
        }
    }
    function get(uint256 key) public returns (uint256 value) {
        assembly {
            key
            sload
            =: value
        }
    }
}
