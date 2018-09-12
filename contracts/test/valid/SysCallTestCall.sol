pragma solidity ^0.4.17;

contract SysCallTestCall {
    // Log to no topics
    function A() public {
        bytes24 reqProc = bytes24("TestWrite");
        assembly {
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(0x0,0x03)
            // The capability index is 0x-02
            mstore(0x20,0x02)
            // The key of the procedure
            mstore(0x40,reqProc)
            // // The value we want to log
            // mstore(0x60,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 65, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,add(1100,mload(0x80)))
            return(0xd,0x20)
        }
    }

    // Call SysCallTes
    function B() public {
        bytes24 reqProc = bytes24("SysCallTest");
        assembly {
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(0x0,0x03)
            // The capability index is 0x-02
            mstore(0x20,0x02)
            // The key of the procedure
            mstore(0x40,reqProc)
            // // The value we want to log
            // mstore(0x60,0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 65, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,add(1100,mload(0x80)))
            return(0xd,0x20)
        }
    }

    // Call SysTestCall:S()
    function C() public {
        bytes24 reqProc = bytes24("SysCallTest");
        string memory fselector = "S()";
        assembly {
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(0x0,0x03)
            // The capability index is 0x-02
            mstore(0x20,0x02)
            // The key of the procedure
            mstore(0x40,reqProc)
            // The data from 0x60 onwards is the data we want to send to
            // this procedure
            // First we store the function selector in the 0x20 bytes from 0x60
            mstore(0x60,keccak256(add(fselector,0x20),mload(fselector)))
            // mstore(0x60,mload(add(fselector,0x20)))
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 69, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,add(1100,mload(0x80)))
            return(0xd,0x20)
        }
    }

    // Call Adder:add(3,5)
    function D() public {
        bytes24 reqProc = bytes24("Adder");
        string memory fselector = "add()";
        assembly {
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(0x0,0x03)
            // The capability index is 0x-02
            mstore(0x20,0x02)
            // The key of the procedure
            mstore(0x40,reqProc)
            // The data from 0x60 onwards is the data we want to send to
            // this procedure
            // First we store the function selector in the 0x20 bytes from 0x60
            mstore(0x60,keccak256(add(fselector,0x20),mload(fselector)))
            mstore(0x64,3)
            mstore(0x84,5)
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+4+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 133, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,add(1100,mload(0x80)))
            return(0xd,0x20)
        }
    }
}