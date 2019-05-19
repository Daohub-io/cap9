pragma solidity ^0.4.17;

contract TestWrite {
    function() public {
        assembly {
            sstore(0x8000,356)
        }
    }
}