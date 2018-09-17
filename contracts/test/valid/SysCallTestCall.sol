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
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            mstore(0x60,retSize)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // mstore(0x80,"hello")
            return(0x80, retSize)
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
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            mstore(0x60,retSize)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            return(0x80, retSize)
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
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            mstore(0x60,retSize)
            // The data from 0x80 onwards is the data we want to send to
            // this procedure
            // First we store the function selector in the 0x20 bytes from 0x60
            mstore(0x80,keccak256(add(fselector,0x20),mload(fselector)))
            // mstore(0x60,mload(add(fselector,0x20)))
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 101, 0x80, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // Return nothing in success
            return(0,0)
        }
    }

    // Call Adder:add(3,5), store, return nothing
    function D() public {
        bytes24 reqProc = bytes24("Adder");
        string memory fselector = "add(uint256,uint256)";
        assembly {
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(0x0,0x03)
            // The capability index is 0x-02
            mstore(0x20,0x02)
            // The key of the procedure
            mstore(0x40,reqProc)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            mstore(0x60,retSize)
            // The data from 0x60 onwards is the data we want to send to
            // this procedure
            // First we store the function selector in the 0x20 bytes from 0x60
            mstore(0x80,keccak256(add(fselector,0x20),mload(fselector)))
            // Then we write the arguments to memory. This will overwrite all
            // but the first 4 bytes of the function selector, which is what we
            // want.
            // first argument
            mstore(0x84,3)
            // second argument
            mstore(0xa4,5)
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 165, 0x80, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            // Store the value in the test location
            sstore(0x8000,mload(0x80))
            // return nothing in success
            return(0, 0)
        }
    }

    // Call Adder:add(3,5), return result
    function E() public {
        bytes24 reqProc = bytes24("Adder");
        string memory fselector = "add(uint256,uint256)";
        assembly {
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(0x0,0x03)
            // The capability index is 0x-02
            mstore(0x20,0x02)
            // The key of the procedure
            mstore(0x40,reqProc)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            mstore(0x60,retSize)
            // The data from 0x60 onwards is the data we want to send to
            // this procedure
            // First we store the function selector in the 0x20 bytes from 0x60
            mstore(0x80,keccak256(add(fselector,0x20),mload(fselector)))
            mstore(0x84,3)
            mstore(0xa4,5)
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 165, 0x80, retSize)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            return(0x80,retSize)
        }
    }

    // Do deeper call stacks
    function F() public returns (uint256) {
        bytes24 reqProc = bytes24("Adder");
        string memory fselector = "add(uint256,uint256)";
        // We will store the result from the first procedure call (add) here
        uint256 addResult;
        assembly {

            function malloc(size) -> result {
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            // allocate some space for data
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            let inl := 165
            let ins := malloc(224)

            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(ins,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(ins,0x20),0x02)
            // The key of the procedure
            mstore(add(ins,0x40),reqProc)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            mstore(add(ins,0x60),retSize)
            // The data from 0x60 onwards is the data we want to send to
            // this procedure
            // First we store the function selector in the 0x20 bytes from 0x60
            mstore(add(ins,0x80),keccak256(add(fselector,0x20),mload(fselector)))
            mstore(add(ins,0x84),3)
            mstore(add(ins,0xa4),5)
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4+32+32
            // we will store the result at 0x80 and it will be 32 bytes

            let retLoc := malloc(retSize)
            if iszero(delegatecall(gas, caller, add(ins,31), inl, retLoc, retSize)) {
                mstore(0xd,add(2200,mload(retLoc)))
                revert(0xd,0x20)
            }
            if iszero(eq(mload(retLoc),8)) {
                mstore(retLoc,77)
                revert(retLoc,0x20)
            }
            mstore(0x999,mload(retLoc))
            addResult := mload(retLoc)

        }
        reqProc = bytes24("SysCallTest");
        string memory newfselector = "S()";
        assembly {
            function malloc(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            // allocate some space for data
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            let inl := 101
            let ins := malloc(160)

            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(ins,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(ins,0x20),0x02)
            // The key of the procedure
            mstore(add(ins,0x40),reqProc)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            let retLoc := malloc(retSize)
            mstore(add(ins,0x60),retSize)
            mstore(add(ins,0x80),keccak256(add(newfselector,0x20),mload(newfselector)))
            // we only want the first 4 bytes of this
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 69 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), inl,  retLoc, retSize)) {
                mstore(0xd,add(2500,mload(retLoc)))
                revert(0xd,0x20)
            }
        }
        return addResult;
    }
}