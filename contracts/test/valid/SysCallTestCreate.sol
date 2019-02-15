pragma solidity ^0.4.17;

contract SysCallTestCreate {
    // Register a procedure
    function A(bytes24 name, address procAddress) public {
        assembly {
            function mallocZero(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // zero-out the memory
                // if there are some bytes to be allocated (rsize is not zero)
                if rsize {
                    // loop through the address and zero them
                    for { let n := 0 } iszero(eq(n, rsize)) { n := add(n, 32) } {
                        mstore(add(result,n),0)
                    }
                }
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            let inSize := add(97,0)
            let ins := mallocZero(inSize)
            // First set up the input data (at memory location 0x0)
            // The register syscall is 4
            mstore(add(ins,0x0),4)
            // The capability index is 0x-01
            mstore(add(ins,0x20),0x01)
            // The name of the procedure (24 bytes)
            mstore(add(ins,0x40),name)
            // The address (20 bytes)
            mstore(add(ins,0x60),procAddress)
            // The caps are just listed one after another, not in the dyn array
            // format specified by Solidity
            // for { let n := 0 } iszero(eq(n, mul(nCapKeys,0x20))) { n := add(n, 0x20) } {
            //     mstore(add(add(ins,0x80),n),add(caps,add(0x20,n)))
            // }
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            let retSize := 0x20
            let retLoc := mallocZero(retSize)
            if iszero(delegatecall(gas, caller, add(ins,31), inSize, retLoc, retSize)) {
                mstore(0xd,add(2200,mload(retLoc)))
                revert(0xd,retSize)
            }
            // We don't need to return anything in success
            return(retLoc,retSize)
        }
    }

    // Register a procedure with capabilities
    function B(bytes24 name, address procAddress, uint256[] caps) public {
        uint256 nCapKeys = caps.length;
        assembly {
            function mallocZero(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // zero-out the memory
                // if there are some bytes to be allocated (rsize is not zero)
                if rsize {
                    // loop through the address and zero them
                    for { let n := 0 } iszero(eq(n, rsize)) { n := add(n, 32) } {
                        mstore(add(result,n),0)
                    }
                }
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            let nCapBytes := mul(nCapKeys,32)
            let inSize := add(97,nCapBytes)
            let ins := mallocZero(inSize)
            // First set up the input data (at memory location 0x0)
            // The register syscall is 4
            mstore(add(ins,0x0),4)
            // The capability index is 0x-01
            mstore(add(ins,0x20),0x01)
            // The name of the procedure (24 bytes)
            mstore(add(ins,0x40),name)
            // The address (20 bytes)
            mstore(add(ins,0x60),procAddress)
            // The caps are just listed one after another, not in the dyn array
            // format specified by Solidity
            for { let n := 0 } iszero(eq(n, mul(nCapKeys,0x20))) { n := add(n, 0x20) } {
                mstore(add(add(ins,0x80),n),mload(add(caps,add(0x20,n))))
            }
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            let retSize := 0x20
            let retLoc := mallocZero(retSize)
            if iszero(delegatecall(gas, caller, add(ins,31), inSize, retLoc, retSize)) {
                mstore(0xd,add(77,mul(100,mload(retLoc))))
                revert(0xd,retSize)
            }
            // We don't need to return anything in success
            return(retLoc,retSize)
        }
    }

    function testNum() public pure returns (uint256) {
        return 392;
    }
}