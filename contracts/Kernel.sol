pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

library WhatIsMyAddress {
    function get() public view returns (address) {
        return msg.sender;
    }
}

contract Kernel is Factory {
    event KernelLog(string message);
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;
    address kernelAddress;

    struct Process {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
    }


    function Kernel() {
        // kernelAddress = WhatIsMyAddress.get();
        // This is an example kernel global variable for testing.
        assembly {
            sstore(0x8000,3)
        }
        // The kernel is inaccessible if there is not init procedure, so we must
        // deploy one.

        // We also need a createProcedure procedure which must be deployed
        // early
    }

    function callGuardProcedure() {

    }

    // This is the fallback function which is used to handle system calls. This
    // is only called if the other functions fail.
    function() public {
        // This is the entry point for the kernel
        Process[] memory processTable = new Process[](256);
        // Before we do anything, let's set up some information about this call.
        // Where is this call coming from? If it is an external account we can
        // just use the caller value.


        // If it is an external account, we forward it straight to the init
        // procedure.
        // if (isExternal) {
        //     callGuardProcedure(msg.sender, msg.data);
        // }


        // 0x00 - not a syscall
        // 0x01 - read syscall
        // 0x02 - write syscall
        // 0x03 - exec syscall
        assembly {
            // Here we decode the system call (if there is one)
            switch div(calldataload(0), 0x10000000000000000000000000000000000000000000000000000000000)
            case 0 {
                // non-syscall case
                // here we need to use callcode. delegatecall would leave the CALLER as
                // the account which started the transaction
                // The address here needs to be updated to call Procedure1
                // everytime Procedure1 is deployed
                //
                // TODO: Determine the address of the procedure at index 0 of
                // the procedure table.
                //
                // This (from last parameters to first):
                // 1. Allocates an area in memory of size 0x40 (64 bytes)
                // 2.    At memory location 0x80
                // 3. Set the input size to 67 bytes
                // 4.    At memory location 29
                // 5. Send 0 wei
                // 6. The contract address we are calling to
                // 7. The gas we are budgeting
                //
                //        7                       6                       5   4   3   2      1
                callcode(gas, 0x8885584aa73fccf0f4572a770d1a0d6bd0b4360a, 0, 29, 67, 0x80, 0x40)
                // store the return code in memory location 0x60 (1 byte)
                0x60
                mstore
                // return both delegatecall return values and the system call
                // retun value
                return(0x60,0x60)
            }
            // This is a store system call
            case 2 {
                // First we need to check the capability is valid.
                // That means getting the capability list for the procedure.
                // let caps = procedures[currentProcedure].capabilities
                let location := calldataload(3)
                let value := calldataload(add(3,32))
                sstore(location, value)
                mstore8(0xb0,3)
                log0(0xb0, 1)
                // sstore(0, div(x, 2))
            }
            // This is the exec system call
            case 0x03 {
                // First we need to check if we have the capability to
                // execute this procedure. The first argument is simply an index
                // of the procedure we want to call (in the procedure table).
                // How do we determine if we have the capability? Perhaps this
                // is not an address, but simply an index into the CList of the
                // process that called this syscall. How do we access that
                // CList? What if it is an account? If it is an account we know
                // the sender. But if it is a procedure the sender is the
                // the kernel and we don't know who is doing the sending.

                // What process is calling this and what capabilities does it
                // have?


                let location := calldataload(3)
                let value := calldataload(add(3,32))
                sstore(location, value)
                mstore8(0xb0,3)
                log0(0xb0, 1)
                // sstore(0, div(x, 2))
            }
            default {
                mstore8(0xb0,5)
                log0(0xb0, 1)
            }

        }
    }

    function createProcedure(bytes24 name, bytes oCode) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name == 0) {
            err = 1;
            return;
        }

        // Check whether the address exists
        bool exist = procedures.contains(name);
        if (exist) {
            err = 3;
            return;
        }

        procedureAddress = create(oCode);
        procedures.insert(name, procedureAddress);
    }

    function deleteProcedure(bytes24 name) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }

        procedureAddress = procedures.get(name);
        bool success = procedures.remove(name);

        // Check whether the address exists
        if (!success) {err = 2;}
    }

    function listProcedures() public view returns (bytes24[] memory) {
        return procedures.getKeys();
    }

    // function nProcedures() public view returns (uint256) {
    //     return procedures.size();
    // }


    function getProcedure(bytes24 name) public returns (address) {
        return procedures.get(name);
    }

    function executeProcedure(bytes24 name, string fselector, bytes payload) public returns (uint8 err, uint256 retVal) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }
        // Check whether the address exists
        bool exist = procedures.contains(name);
        if (!exist) {
            err = 3;
            return;
        }

        address procedureAddress = procedures.get(name);
        bool status = false;
        assembly {
            // Retrieve the address of new available memory from address 0x40
            let n :=  mload(0x40)
            // Replace the value of 0x40 with the next new available memory,
            // after the 4 bytes we will use to store the keccak hash.
            mstore(0x40,add(n,32))
            // Take the keccak256 hash of that string, store at location n
            // mstore
            // Argument #1: The address (n) calculated above, to store the
            //    hash.
            // Argument #2: The hash, calculted as follows:
            //   keccack256
            //   Argument #1: The location of the fselector string (which
            //     is simply the name of the variable) with an added offset
            //     of 0x20, as the first 0x20 is reserved for the length of
            //     the string.
            //   Argument #2: The length of the string, which is loaded from
            //     the first 0x20 of the string.
            mstore(n,keccak256(add(fselector,0x20),mload(fselector)))

            // The input starts at where we stored the hash (n)
            let ins := n
            // Currently that is only the function selector hash, which is 4
            // bytes long.
            let inl := 0x4
            // TODO: Allocate a new area of memory into which to write the
            // return data. This will depend on the size of the return type.
            let outs := 0x60
            // There is no return value, therefore it's length is 0 bytes long
            // REVISION: lets assume a 32 byte return value for now
            let outl := 0x20

            status := callcode(gas,procedureAddress,0,ins,inl,outs,outl)
            if eq(status,0) {
                // error
                mstore(0x40,4)
                return(0x40,0x40)
            }
            if eq(status,1) {
                mstore(0x40,0)
                return(0x40,0x40)
            }
        }
        if (!status) {
            err = 4;
        }
    }
}