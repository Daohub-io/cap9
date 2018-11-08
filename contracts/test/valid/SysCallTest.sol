pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTest is BeakerContract {
    function S() public {
        uint256 n = read(0x8000);
        write(0x8000,n+1);
        assembly {
            if iszero(eq(sload(0x8000),add(n,1))) {
                mstore(0xd,2200)
                revert(0xd,0x20)
            }
            mstore(0x80,0)
            return(0x80,0)
        }
    }

    function() public {
        write(0x8000,356);
        assembly {
            mstore(0x99,0)
            return(0x99,0)
        }
    }
}