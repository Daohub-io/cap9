pragma solidity ^0.4.17;

contract ThirdNestedCall {


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
            // First get the original value from storage
            let orig_value := sload(0x8003)
            // First set up the input data (at memory location 0x0)
            // The write call is 0x-07
            mstore(0x0,0x07)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The storage location we want is 0x8000
            mstore(0x40,0x8003)
            // The value we want to store
            mstore(0x60,add(orig_value,1))
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, 31, 97, 0x80, 0x20)) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
            mstore(0xd,sload(0x8003))
            if iszero(eq(sload(0x8003),add(orig_value,1))) {
                mstore(0xd,add(2200,mload(0x80)))
                revert(0xd,0x20)
            }
        }
        // End of write call
        // Being our call to FourthNestedCall
        bytes24 reqProc = bytes24("FourthNestedCall");
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
        }
        // End procedure call
        // TODO: perform some checks and return
    }
}