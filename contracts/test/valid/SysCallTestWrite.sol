pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestWrite is BeakerContract {
    function S() public returns (uint8 error) {
        uint256 n = read(0x8000);
        return write(1, 0x8000,n+1);
    }

    function() public {
        uint8 err = write(1, 0x8000,356);
        assembly {
            mstore(0x80,err)
            return(0x80,0x20)
        }
    }
}