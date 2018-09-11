pragma solidity ^0.4.17;

contract Instance {

    // TODO: All variables should be located in designated kernel space
    address kernelAddress;
    bytes24 currentProcedure;

    // Here we construct a new kernel instance
    // If kernel address is zero, we assume the sender the from the kernel lib itself (Kernel#new_instance)
    constructor(address _kernelAddress, address _entryProcedureAddress, bytes32 _entryProcedureId ) public {
        kernelAddress = _kernelAddress == 0 ? msg.sender: _kernelAddress;
    }

    // This is what we execute when we receive an external transaction.
    function callExternal() internal {
        // Here we need to use callcode. delegatecall would leave the CALLER as
        // the account which started the transaction
        // The address here needs to be updated to call Procedure1
        // everytime Procedure1 is deployed
        //
        // This (from last parameters to first):
        // 1. Set the output size to size of memory
        // 2. At memory location 0
        // 3. Set the input size to size of memory
        // 4. At memory location 0
        // 5. The contract address we are calling to
        // 6. The gas we are budgeting
        //
        assembly {
            //        7            6          5  4  3      2  1
            callcode(gas, kernelAddress_slot, 0, 0, msize, 0, msize)
            
            // store the return code in memory location 0x60 (1 byte)
            0x60
            mstore
            // return both delegatecall return values and the system call
            // return value
            return(0x60,0x60)
        }
    }

    // This is what we execute when we receive an internal transaction
    function callSyscall() internal {

        // This (from last parameters to first):
        // 1. Allocates an area in memory of size 0x40 (64 bytes)
        // 2. At memory location 0x80
        // 3. Set the input size to 67 bytes
        // 4. At memory location 29
        // 5. The kernel library address
        // 6. The gas we are budgeting
        assembly {
            //            6            5          4   3   2     1
            delegatecall(gas, kernelAddress_slot, 29, 67, 0x80, 0x40)
            // store the return code in memory location 0x60 (1 byte)
            0x60
            mstore
            // return both delegatecall return values and the system call
            // return value
            return(0x60,0x60)
        }
    }
    
    // Check if a transaction is internal.
    function isInternal() internal view returns (bool) {
        // Check if the CALLER is from the instance
        return (address(this) == msg.sender);
    }

    function() public {
        if (isInternal()) {
            callSyscall();
        } else {
            callExternal();
        }
    }
}
