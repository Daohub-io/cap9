pragma solidity ^0.4.17;

contract Suicide {
    function foo(address a) public {
        selfdestruct(a);
    }
}
