pragma solidity ^0.4.17;

contract Create {
    function foo(byte value, byte offset, byte size) public {
        assembly {
            let x := create(value, offset, size)
        }
    }
}
