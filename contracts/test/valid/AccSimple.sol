pragma solidity ^0.4.17;

contract AccSimple {

    function () public payable {
        assembly {
            mstore(0,37)
            return(0,32)
        }
    }

    function A() public pure {

    }

    function B() public pure {

    }

    function C(uint256 /* a */) public pure {

    }

}
