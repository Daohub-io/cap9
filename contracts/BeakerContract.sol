pragma solidity ^0.4.17;

import "./Kernel.sol";

contract BeakerContract is IKernel {

    // TODO: this doesn't actually use caps, just reads raw
    function read(uint256 location) public view returns (uint256 result) {
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
        // The write call is 0x-07
        mstore(add(ins,0x0),0x07)
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

  function set_entry(uint8 capIndex, bytes32 procId) internal returns (uint32 err) {
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

        let ins := mallocZero(0x60)
        // First set up the input data (at memory location 0x0)
        // The delete syscall is 6
        mstore(add(ins,0x0),6)
        // The capability index
        mstore(add(ins,0x20),capIndex)
        // The name of the procedure (24 bytes)
        mstore(add(ins,0x40),procId)
        // "in_offset" is at 31, because we only want the last byte of type
        // "in_size" is 65 because it is 1+32+32
        // we will store the result at 0x80 and it will be 32 bytes
        let retSize := 0x20
        let retLoc := mallocZero(retSize)
        err := 0
        if iszero(delegatecall(gas, caller, add(ins,31), 65, retLoc, retSize)) {
            err := add(2200, mload(retLoc))
            mstore(0xd, err)
            revert(0xd,retSize)
        }
        return(retLoc, retSize)
    }
    return err;
  }

  function proc_call(uint8 capIndex, bytes32 procId, string fselector, uint32[] input) internal returns (uint32 err, bytes memory output) {
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

            // We Get The start of the Proc Input
            // Then allocate data to include it
            // let pInputs := add(input, 0x20)
            // let inSize := add(mload(input), 96)

            let inputSize := mul(mload(input), 0x20)
            let bufSize := add(0x80, inputSize)

            // If fselector is non-empty
            let fselSize := mload(fselector)
            if fselSize { bufSize := add(bufSize, 0x20)}

            let buf := mallocZero(bufSize)

            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(buf,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(buf,0x20),capIndex)
            // The key of the procedure
            mstore(add(buf,0x40),procId)

            // The data from 0x80 onwards is the data we want to send to
            // this procedure
            let inputStart := add(input, 0x20)
            let bufStart := add(buf, 0x80)

            // If selector is non-empty, add it
            if fselSize {
                mstore(bufStart, keccak256(add(fselector, 0x20), fselSize))
                bufStart := add(bufStart, 4)
            }

            for { let n:= 0 } iszero(eq(n, inputSize)) { n := add(n, 32)} {
                mstore(add(bufStart, n), mload(add(inputStart, n)))
            }

            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(buf,31), sub(bufSize, 31), 0, 0)) {
                let outSize := returndatasize
                output := malloc(add(outSize, 0x20))
                mstore(output, outSize)

                returndatacopy(add(output, 0x20), 0, outSize)
                err := add(2200, mload(add(output, 0x20)))

                mstore(add(output, 0x20), err)
                revert(add(output, 0x20),outSize)
            }

            // simply return whatever the system call returned
            let outSize := returndatasize
            output := malloc(add(outSize, 0x20))
            mstore(output, outSize)

            returndatacopy(add(output, 0x20), 0, outSize)
            err := 0
        }
  }

  function proc_reg(uint8 capIndex, bytes32 procId, address procAddr, uint256[] caps) internal returns (uint32 err) {
    uint256 nCapKeys = caps.length;
    bytes memory input = new bytes(97 + nCapKeys*32);
    uint256 inSize = input.length;
    bytes memory retInput = new bytes(32);
    uint256 retSize = retInput.length;

    assembly {
        let ins := add(input, 0x20)
        // First set up the input data (at memory location 0x0)
        // The register syscall is 4
        mstore(add(ins,0x0),4)
        // The capability index is 0x-01
        mstore(add(ins,0x20),capIndex)
        // The name of the procedure (24 bytes)
        mstore(add(ins,0x40),procId)
        // The address (20 bytes)
        mstore(add(ins,0x60),procAddr)
        // The caps are just listed one after another, not in the dyn array
        // format specified by Solidity
        for { let n := 0 } iszero(eq(n, mul(nCapKeys,0x20))) { n := add(n, 0x20) } {
            mstore(add(add(ins,0x80),n),mload(add(caps,add(0x20,n))))
        }
        // "in_offset" is at 31, because we only want the last byte of type
        // "in_size" is 97 because it is 1+32+32+32
        // we will store the result at 0x80 and it will be 32 bytes
        let retLoc := add(retInput, 0x20)
        err := 0
        if iszero(delegatecall(gas, caller, add(ins,31), inSize, retLoc, retSize)) {
            err := add(2200, mload(retLoc))
            if nCapKeys {
                err := add(77,mul(100,mload(retLoc)))
            }
            mstore(0xd, err)
            revert(0xd,retSize)
        }
    }
    return err;
  }

  function proc_del(uint8 capIndex, bytes32 procId) internal returns (uint32 err) {
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

        let ins := mallocZero(0x60)
        // First set up the input data (at memory location 0x0)
        // The delete syscall is 5
        mstore(add(ins,0x0),5)
        // The capability index
        mstore(add(ins,0x20),capIndex)
        // The name of the procedure (24 bytes)
        mstore(add(ins,0x40),procId)
        // "in_offset" is at 31, because we only want the last byte of type
        // "in_size" is 65 because it is 1+32+32
        // we will store the result at 0x80 and it will be 32 bytes
        let retSize := 0x20
        let retLoc := mallocZero(retSize)
        err := 0
        if iszero(delegatecall(gas, caller, add(ins,31), 65, retLoc, retSize)) {
            err := add(2200, mload(retLoc))
            mstore(0xd, err)
            revert(0xd,retSize)
        }
        return(retLoc, retSize)
    }
    return err;
  }

  function proc_log0(uint8 capIndex, uint32 value) internal returns (uint32 err) {
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
            // The log call is 0x-08
            mstore(add(ins,0x0),0x08)
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
  function proc_log1(uint8 capIndex, uint32 t1, uint32 value) internal returns (uint32 err) {
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
            // The log call is 0x-08
            mstore(add(ins,0x0),0x08)
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

  function proc_log2(uint8 capIndex, uint32 t1, uint32 t2, uint32 value) internal returns (uint32 err) {
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
            // The log call is 0x-08
            mstore(add(ins,0x0),0x08)
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
  function proc_log3(uint8 capIndex, uint32 t1, uint32 t2, uint32 t3, uint32 value) internal returns (uint32 err) {
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
            // The log call is 0x-08
            mstore(add(ins,0x0),0x08)
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

  function proc_log4(uint8 capIndex, uint32 t1, uint32 t2, uint32 t3, uint32 t4, uint32 value) internal returns (uint32 err) {
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
            // The log call is 0x-08
            mstore(add(ins,0x0),0x08)
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
