pragma solidity ^0.4.17;

import "./Kernel.sol";

contract BeakerContract is IKernel {
    
    function read(uint256 location) public view returns (uint256 result) {
        // TODO: this doesn't actually use caps, just reads raw
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
            let retSize := 0x20
            let retLoc := mallocZero(retSize)
            mstore(retLoc,sload(location))
            result := sload(location)
        }
        return result;
    }
  
  /// Returns 0 on success, 1 on error
  function write(uint8 capIndex, uint256 location, uint256 value) internal returns (uint8 err) {
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
        let inSize := 0x80
        let ins := mallocZero(inSize)
        // First set up the input data (at memory location 0x0)
        // The write call is 0x-08
        mstore(add(ins,0x0),0x08)
        // The capability index is 0x-01
        mstore(add(ins,0x20),capIndex)
        // The storage location we want is 0x8000
        mstore(add(ins,0x40),location)
        // The value we want to store
        mstore(add(ins,0x60),value)
        // clear the output buffer
        let retSize := 0x20
        let retLoc := mallocZero(retSize)
        // "in_offset" is at 31, because we only want the last byte of type
        // "in_size" is 97 because it is 1+32+32+32
        // we will store the result at retLoc and it will be 32 bytes
        if iszero(delegatecall(gas, caller, add(ins,31), 97, retLoc, retSize)) {
            mstore(retLoc,0)
            revert(retLoc,retSize)
        }
        err := mload(retLoc)

        // Free Memory
        mstore(0x40, ins)
    }
    return err;
  }
  
  function log0(uint8 capIndex, uint32 value) internal returns (uint32 err) {
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
            let ins := mallocZero(mul(4,32))
            let retSize := 0x20
            let retLoc := mallocZero(retSize)
            // First set up the input data (at memory location ins)
            // The log call is 0x-09
            mstore(add(ins,0x0),0x09)
            // The capability index is 0x-01
            mstore(add(ins,0x20),capIndex)
            // The number of topics we will use
            mstore(add(ins,0x40),0x0)
            // The value we want to log
            mstore(add(ins,0x60),value)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 97, retLoc, retSize)) {
                mstore(retLoc,add(2200,mload(retLoc)))
                revert(retLoc,retSize)
            }
            err := mload(retLoc)

            // Free Memory
            mstore(0x40, ins)
        }
        return err;
  }
  function log1(uint8 capIndex, uint32 t1, uint32 value) internal returns (uint32 err) {
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
            let ins := mallocZero(mul(5,32))
            let retSize := 0x20
            let retLoc := mallocZero(retSize)

            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(add(ins,0x0),0x09)
            // The capability index is 0x-01
            mstore(add(ins,0x20),capIndex)
            // The number of topics we will use
            mstore(add(ins,0x40),0x1)
            // The first topic
            mstore(add(ins,0x60),t1)
            // The value we want to log
            mstore(add(ins,0x80),value)

            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 129 because it is 1+32+32+32+32
            // we will store the result at retLoc and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 129, retLoc, retSize)) {
                mstore(retLoc,add(2200,mload(retLoc)))
                revert(retLoc,retSize)
            }
            err := mload(retLoc)

            // Free Memory
            mstore(0x40, ins)
        }
        return err;
  }

  function log2(uint8 capIndex, uint32 t1, uint32 t2, uint32 value) internal returns (uint32 err) {
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
            let ins := mallocZero(mul(6,32))
            let retSize := 0x20
            let retLoc := mallocZero(retSize)
          
            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(add(ins,0x0),0x09)
            // The capability index is 0x-01
            mstore(add(ins,0x20),capIndex)
            // The number of topics we will use
            mstore(add(ins,0x40),0x2)
            // The first topic
            mstore(add(ins,0x60),t1)
            // The second topic
            mstore(add(ins, 0x80),t2)
            // The value we want to log
            mstore(add(ins, 0xa0),value)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 161 because it is 1+32+32+32+32+32
            // we will store the result at retLoc and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 161, retLoc, retSize)) {
                mstore(retLoc,add(2200,mload(retLoc)))
                revert(retLoc,retSize)
            }
            err := mload(retLoc)

            // Free Memory
            mstore(0x40, ins)
        }
    return err;
  }
  function log3(uint8 capIndex, uint32 t1, uint32 t2, uint32 t3, uint32 value) internal returns (uint32 err) {
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
            let ins := mallocZero(mul(7,32))
            let retSize := 0x20
            let retLoc := mallocZero(retSize)

            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(add(ins,0x0),0x09)
            // The capability index is 0x-01
            mstore(add(ins,0x20),capIndex)
            // The number of topics we will use
            mstore(add(ins,0x40),0x3)
            // The first topic
            mstore(add(ins,0x60),t1)
            // The second topic
            mstore(add(ins,0x80),t2)
            // The third topic
            mstore(add(ins,0xa0),t3)
            // The value we want to log
            mstore(add(ins,0xc0),0x1234567890)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 193 because it is 1+32+32+32+32+32+32
            // we will store the result at retLoc and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 193, retLoc, retSize)) {
                mstore(retLoc,add(2200,mload(retLoc)))
                revert(retLoc,retSize)
            }
            err := mload(retLoc)

            // Free Memory
            mstore(0x40, ins)
        }
        return err;
  }

  function log4(uint8 capIndex, uint32 t1, uint32 t2, uint32 t3, uint32 t4, uint32 value) internal returns (uint32 err) {
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
            let ins := mallocZero(mul(8,32))
            let retSize := 0x20
            let retLoc := mallocZero(retSize)

            // First set up the input data (at memory location 0x0)
            // The log call is 0x-09
            mstore(add(ins,0x0),0x09)
            // The capability index is 0x-01
            mstore(add(ins,0x20),0x01)
            // The number of topics we will use
            mstore(add(ins,0x40),0x4)
            // The first topic
            mstore(add(ins,0x60),t1)
            // The second topic
            mstore(add(ins,0x80),t2)
            // The third topic
            mstore(add(ins,0xa0),t3)
            // The fourth topic
            mstore(add(ins,0xc0),t4)
            // The value we want to log
            mstore(add(ins,0xe0),value)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 225 because it is 1+32+32+32+32+32+32+32
            // we will store the result at retLoc and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 225, retLoc, retSize)) {
                mstore(retLoc,add(2200,mload(retLoc)))
                revert(retLoc,retSize)
            }
            err := mload(retLoc)

            // Free Memory
            mstore(0x40, ins)
        }
        return err;
  }


}