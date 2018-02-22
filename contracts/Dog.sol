pragma solidity ^0.4.17;

contract Dog {
    string public name = "happy";

    function say() public returns (string) {
        return "woof";
    }
}