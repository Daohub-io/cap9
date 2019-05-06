pragma solidity ^0.4.17;

contract Simple {
    function A() public pure {

    }

    function B() public {
        selfdestruct(0);
    }

    function C(uint256 /* a */) public pure {

    }

}