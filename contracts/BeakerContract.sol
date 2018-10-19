pragma solidity ^0.4.17;

contract BeakerContract {

    function read(uint256 location) public view returns (uint256) {
        // TODO: this doesn't actually use caps, just reads raw
        uint256 result;
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

  function write(uint256 location, uint256 value) public returns (bool) {
      bool result;
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
        // The write call is 0x-07
        mstore(add(ins,0x0),0x07)
        // The capability index is 0x-01
        mstore(add(ins,0x20),0x01)
        // The storage location we want is 0x8000
        mstore(add(ins,0x40),location)
        // The value we want to store
        mstore(add(ins,0x60),value)
        // clear the output buffer
        let retSize := 0x20
        let retLoc := mallocZero(retSize)
        // "in_offset" is at 31, because we only want the last byte of type
        // "in_size" is 97 because it is 1+32+32+32
        // we will store the result at 0x80 and it will be 32 bytes
        if iszero(delegatecall(gas, caller, add(ins,31), 97, retLoc, retSize)) {
            mstore(retLoc,0)
            revert(retLoc,retSize)
        }
        result := 1
    }
    return result;
  }
}