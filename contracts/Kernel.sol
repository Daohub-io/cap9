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
    bytes24 currentProcedure;

    struct Process {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
    }

    // All of the data that is sent as a system call. Currently this is fairly
    // hard coded to write.
    // @Jacob, I was looking at how you do things like Sum types or inheritance
    // and all the techniques look like they have implications like being
    // in separate contents. I'll leave this for now to hear your thoughts.
    struct SystemCall {
        // This is the most structure we can define in general
        uint8 capType;
        uint256[] values;
    }

    function Kernel() public {
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

    // Parse the system call from msg.data
    function parseSystemCall() internal pure returns (SystemCall) {
        SystemCall memory syscall;
        // The cap type is the first byte
        syscall.capType = uint8(msg.data[0]);
        uint256 nKeys = (msg.data.length-1)/32;
        syscall.values = new uint256[](nKeys);
        for (uint256 i = 0; i < nKeys; i++) {
            syscall.values[i] = parse32ByteValue(1+i*32);
        }

        return syscall;
    }

    function parse32ByteValue(uint256 startOffset) pure returns (uint256) {
        uint256 value = 0;
        for (uint256 i = 0; i < 32; i++) {
            value = value << 8;
            value = value | uint256(msg.data[startOffset+i]);
        }
        return value;

    }

    // Check if a transaction is external.
    function isExternal() internal view returns (bool) {
        // If the current transaction is from the procedure we are executing,
        // then it is a systemcall. Otherwise it is an external transaction.
        // We have in storage a value (currentProcedure) which states which
        // procedure the kernel is currently executing. If we have this value
        // then something is running, so it must be an internal transaction.
        // If it is empty, it means we are not executing anything, and it is an
        // external transaction.

        // While the kernel is executing a procedure, nothing else can run
        // across the whole blockchain, therefore we must be receiving a
        // transaction from the procedure set in "currentProcedure"


        // TODO: we will need to reserve a value for "not executing anything"
        // If the transaction is from this procedure...
        return (currentProcedure == 0);

    }

    // This is what we execute when we receive an external transaction.
    function callGuardProcedure(address sender, bytes data) internal {
        // revert("external call");
        // TODO: this is not currerntly in any code path because we just use
        // "executeProcedure"
        // here we need to use callcode. delegatecall would leave the CALLER as
        // the account which started the transaction
        // The address here needs to be updated to call Procedure1
        // everytime Procedure1 is deployed
        //
        // TODO: Determine the address of the procedure at index 0 of
        // the procedure table.
        // This (from last parameters to first):
        // 1. Allocates an area in memory of size 0x40 (64 bytes)
        // 2.    At memory location 0x80
        // 3. Set the input size to 67 bytes
        // 4.    At memory location 29
        // 5. Send 0 wei
        // 6. The contract address we are calling to
        // 7. The gas we are budgeting
        //
        assembly {
            //        7                       6                       5   4   3   2      1
            callcode(gas, 0x8885584aa73fccf0f4572a770d1a0d6bd0b4360a, 0, 29, 67, 0x80, 0x40)
            // store the return code in memory location 0x60 (1 byte)
            0x60
            mstore
            // return both delegatecall return values and the system call
            // return value
            return(0x60,0x60)
        }
    }

    // This is the fallback function which is used to handle system calls. This
    // is only called if the other functions fail.
    function() public {
        bool cap;
        uint256 capIndex;
        // This is the entry point for the kernel
        // TODO: we aren't currently using this, as we can't invoke a cap that
        // calls other procedures.
        Process[] memory processTable = new Process[](256);
        // Before we do anything, let's set up some information about this call.
        // Where is this call coming from? If it is an external account we can
        // just use the caller value.
        // TODO: we will implement this when we stop using "executeProcedure"


        // If it is an external account, we forward it straight to the init
        // procedure. We currently shouldn't reach this point, as we usually use
        // "executeProcedure"
        if (isExternal()) {
            callGuardProcedure(msg.sender, msg.data);
        }

        // Parse the system call
        SystemCall memory syscall = parseSystemCall();

        // 0x00 - not a syscall
        // 0x01 - read syscall
        // 0x02 - write syscall
        // 0x03 - exec syscall

        // Here we decode the system call (if there is one)
        if (syscall.capType == 0) {
            // non-syscall case
        } else if (syscall.capType == 0x07) {
            // This is a store system call
            // Here we have established that we are processing a write call and
            // we must destructure the necessary values.
            capIndex = syscall.values[0];
            uint256 writeAddress = syscall.values[1];
            uint256 writeValue = syscall.values[2];
            cap = procedures.checkWriteCapability(uint192(currentProcedure), writeAddress, capIndex);
            if (cap) {
                assembly {
                    sstore(writeAddress, writeValue)
                    mstore(0,11)
                    return(0,0x20)
                }
            } else {
                assembly {
                    mstore(0,22)
                    revert(0,0x20)
                }
            }
        } else if (syscall.capType == 0x09) {
            // This is a log system call
            // Here we have established that we are processing a write call and
            // we must destructure the necessary values.
            capIndex = syscall.values[0];
            uint256 nTopics = syscall.values[1];
            bytes32[] memory topicVals = new bytes32[](nTopics);
            for (uint256 i = 0; i < nTopics; i++) {
                topicVals[i] = bytes32(syscall.values[2+i]);
            }
            bytes32 logValue = bytes32(syscall.values[2+nTopics]);
            cap = procedures.checkLogCapability(uint192(currentProcedure), capIndex);
            if (cap) {
                if (nTopics == 0) {
                    log0(logValue);
                    assembly {
                        mstore(0,11)
                        return(0,0x20)
                    }
                } else if (nTopics == 1) {
                    log1(logValue, topicVals[0]);
                    assembly {
                        mstore(0,11)
                        return(0,0x20)
                    }
                } else if (nTopics == 2) {
                    log2(logValue, topicVals[0], topicVals[1]);
                    assembly {
                        mstore(0,11)
                        return(0,0x20)
                    }
                } else if (nTopics == 3) {
                    log3(logValue, topicVals[0], topicVals[1], topicVals[2]);
                    assembly {
                        mstore(0,11)
                        return(0,0x20)
                    }
                } else if (nTopics == 4) {
                    log4(logValue, topicVals[0], topicVals[1], topicVals[2], topicVals[3]);
                    assembly {
                        mstore(0,11)
                        return(0,0x20)
                    }
                } else {
                    assembly {
                        mstore(0,44)
                        revert(0,0x20)
                    }
                }
            } else {
                assembly {
                    mstore(0,33)
                    revert(0,0x20)
                }
            }
            assembly{
                mstore(0xd,152)
                return(0xd,0x20)
            }
        } else if (syscall.capType == 0x03) {
            // This is the exec system call
            //
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

            // TODO: implement
        } else {
            // default; fallthrough action
            assembly {
                mstore8(0xb0,5)
                log0(0xb0, 1)
            }
        }
    }

    function createProcedure(bytes24 name, bytes oCode, uint256[] caps) public returns (uint8 err, address procedureAddress) {
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
        procedures.insert(name, procedureAddress, caps);
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

    function returnRawProcedureTable() public view returns (uint256[]) {
        return procedures.returnRawProcedureTable();
    }

    function returnProcedureTable() public view returns (uint256[]) {
        return procedures.returnProcedureTable();
    }

    // function nProcedures() public view returns (uint256) {
    //     return procedures.size();
    // }


    function getProcedure(bytes24 name) public returns (address) {
        return procedures.get(name);
    }

    function executeProcedure(bytes24 name, string fselector, bytes payload) public returns (uint256 retVal) {
        // // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            retVal = 1;
            return;
        }
        // Check whether the address exists
        bool exist = procedures.contains(name);
        if (!exist) {
            retVal = 3;
            return;
        }
        // TODO: I believe this should use the keyindex
        currentProcedure = name;
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
            sstore(currentProcedure_slot,0)
            if eq(status,0) {
                // error
                mstore(0x0,add(220000,mload(outs)))
                // mstore(0x40,235)
                // TODO: switch to revert
                return(0x0,0x20)
            }
            if eq(status,1) {
                mstore(0x0,add(110000,mload(outs)))
                // mstore(0x40,235)
                // TODO: switch to revert
                return(0x0,0x20)
            }
        }
        if (!status) {
            retVal = 85;
        }
    }
}