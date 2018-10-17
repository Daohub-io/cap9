pragma solidity ^0.4.17;

contract SysCallTestCreate {
    // Register a procedure
    function A(bytes24 name, address procAddress) public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The register syscall is 11
            mstore(0,11)
            // The capability index is 0x-01
            mstore(0x20,0x01)
            // The name of the procedure (24 bytes)
            mstore(0x40,name)
            // The address (20 bytes)
            mstore(0x60,procAddress)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 77 because it is 1+32+32+32
            // we will store the result at 0x80 and it will be 32 bytes
            let retLoc := 0x120
            let retSize := 0x20
            if iszero(delegatecall(gas, caller, 31, 97, retLoc, retSize)) {
                mstore(0xd,add(2200,mload(retLoc)))
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