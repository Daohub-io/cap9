pragma solidity ^0.4.17;
// this contract is inline, and all the jumps are statically resolved before injection
contract StorerProtectedInline {
    function store() public {
        uint256 loo = 1234;
        uint256 lowerLimit = 0x0100000000000000000000000000000000000000000000000000000000000000;
        uint256 upperLimit = 0x0200000000000000000000000000000000000000000000000000000000000000;
        assembly {
                mload(loo) // value to store
                0x0100000100000000000000000000000000000000000000000000000000000000 // address to store
                // This code is necessary in front of an SSTORE to pass verification
            // injected store code
                0x0100000000000000000000000000000000000000000000000000000000000000 // lower limit
                dup2 // duplicate store address for comparison
                lt // see if address is lower than the lower limit
                0x0200000000000000000000000000000000000000000000000000000000000000 // upper limit
                dup3 // duplicate store address for comparison
                gt // see if the store address is higher than the upper limit
                or // set top of stack to 1 if either is true
                pc // push the program counter to the stack, this is guaranteed to be an invalid jump destination
                jumpi // jump if the address is out of bounds, the current address on the stack is guaranteed to be invliad and will throw an error
                sstore // perform the store
        }
    }
}
