pragma solidity ^0.4.17;

contract SixthNestedCall {
     // FirstNestedCall - store at 0x8001
     //   SecondNestedCall - store at 0x8002
     //     ThirdNestedCall - store at 0x8003
     //       FourthNestedCall - store at 0x8004
     //     FifthNestedCall - store at 0x8005
     //   SixthNestedCall - store at 0x8006
     // End
    function G() public {
        // First we do the store for FirstNestedCall
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
            function storeCall(capIndex, storeLoc, storeVal) -> retLoc {
                let ins := malloc(128)
                // First set up the input data (at memory location 0x0)
                // The write call is 0x-07
                mstore(add(ins,0x0),0x07)
                // The capability index is 0x-01
                mstore(add(ins,0x20),capIndex)
                // The storage location we want is storeLoc
                mstore(add(ins,0x40),storeLoc)
                // The value we want to store
                mstore(add(ins,0x60),storeVal)
                let retSize := 0x20
                retLoc := malloc(retSize)
                // "in_offset" is at 31, because we only want the last byte of type
                // "in_size" is 97 because it is 1+32+32+32
                // we will store the result at 0x80 and it will be 32 bytes
                if iszero(delegatecall(gas, caller, add(ins,31), 97, retLoc, retSize)) {
                    mstore(0xd,add(2200,mload(0x80)))
                    revert(0xd,0x20)
                }
            }
            storeCall(1, 0x8006, 80)
            pop
        }
        // End of write call
        // TODO: perform some checks and return
    }
}