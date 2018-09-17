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

    function Kernel() public {
        // kernelAddress = WhatIsMyAddress.get();
        // This is an example kernel global variable for testing.
        assembly {
            sstore(0x8000,3)
        }
    }

    function testGetter() public view returns(uint256) {
        assembly {
            mstore(0,sload(0x8000))
            return(0,0x20)
        }
    }

    function testSetter(uint256 value) public {
        assembly {
            sstore(0x8000,value)
        }
    }

    function parse32ByteValue(uint256 startOffset) pure internal returns (uint256) {
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

        // If it is an external account, we forward it straight to the init
        // procedure. We currently shouldn't reach this point, as we usually use
        // "executeProcedure"
        if (isExternal()) {
            callGuardProcedure(msg.sender, msg.data);
        }

        // Parse the system call
        // SystemCall memory syscall = parseSystemCall();
        uint8 sysCallCapType = uint8(msg.data[0]);

        // 0x00 - not a syscall
        // 0x01 - read syscall
        // 0x03 - exec syscall
        // 0x07 - write syscall
        // 0x09 - log syscall

        // TODO: this was used to track some things but conflicts with some
        // tests
        // log1(bytes32(currentProcedure), bytes32("current-procedure"));

        // Here we decode the system call (if there is one)
        if (sysCallCapType == 0) {
            // non-syscall case
        } else if (sysCallCapType == 0x03) {
            // This is a call system call
            // parse a 32-byte value at offset 1 (offset 0 is the capType byte)
            capIndex = parse32ByteValue(1);
            // parse a 32-byte value at offset 1 (offset 0 is the capType byte,
            // offset 1 is the capIndex (32 bytes)
            // We also perform a shift as this is 24 byte value, not a 32 byte
            // value
            bytes24 procedureKey = bytes24(parse32ByteValue(1+32)/0x10000000000000000);
            uint256 returnLength = uint256(parse32ByteValue(1+32*2));
            uint256 dataLength;
            // log1(bytes32(msg.data.length), bytes32("msg.data.length"));
            if (msg.data.length > (1+3*32)) {
                dataLength = msg.data.length - (1+3*32);
            } else {
                dataLength = 0;
            }
            cap = procedures.checkCallCapability(uint192(currentProcedure), procedureKey, capIndex);
            address procedureAddress = procedures.get(procedureKey);
            // Note the procedure we are currently running, we will put this
            // back into the "currentProcedure" after we have finished the call.
            bytes24 previousProcedure = currentProcedure;
            // We set the value for the current procedure in the kernel so that
            // it knows which procedure it is executing (this is important for
            // looking up capabilities).
            currentProcedure = procedureKey;
            if (cap) {
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
                        let n := 0
                        jumpi(loopend, eq(rsize, 0))
                        loop:
                            mstore(add(result,n),0)
                            n := add(n, 32)
                            jumpi(loop, iszero(eq(n, rsize)))
                        loopend:
                        // Bump the value of 0x40 so that it holds the next
                        // available memory location.
                        mstore(0x40,add(result,rsize))
                    }

                    // Retrieve the address of new available memory from address 0x40
                    // we will use this as the start of the input (ins)
                    let ins
                    let inl
                    if dataLength {
                        // If there is any data associated with this procedure
                        // call (this inlcudes the data such as a function
                        // selector) we need to set that as the input data to
                        // the delegatecall.
                        // First we must allocate some memory.
                        ins :=  malloc(dataLength)
                        // Then we store that data at this allocated memory
                        // location
                        calldatacopy(ins, 97, dataLength)
                        inl := dataLength
                        // log0(ins,inl)
                    }
                    if iszero(dataLength) {
                        // If there is not data to be sent we just set the
                        // location and length of the input data to zero. The
                        // location doesn't actually matter as long the length
                        // is zero.
                        ins := 0
                        inl := 0
                    }
                    let retLoc := mallocZero(returnLength)
                    let status := delegatecall(
                        // The gas we are budgeting, which is always all the
                        // available gas
                        gas,
                        // The address for the chosen procedure which we
                        // obtained earlier
                        procedureAddress,
                        // The starting memory offset of the innput data
                        ins,
                        // The length of the input data
                        inl,
                        // TODO: Allocate a new area of memory into which to
                        // write the return data. This will depend on the size
                        // of the return type.
                        // The starting memory offset to place the output data
                        retLoc,
                        // The length of the output data
                        returnLength)
                    // We need to restore the previous procedure as the current
                    // procedure, this can simply be on the stack
                    // sstore(currentProcedure_slot,previousProcedure)
                    sstore(currentProcedure_slot,div(previousProcedure,exp(0x100,8)))
                    if iszero(status) {
                        let errStore := malloc(0x20)
                        mstore(errStore,add(22,mload(retLoc)))
                        // mstore(0x40,235)
                        // TODO: switch to revert
                        revert(errStore,0x20)
                    }
                    if eq(status,1) {
                        return(retLoc,returnLength)
                    }
                }
            } else {
                assembly {
                    // 33 means the capability was rejected
                    mstore(0,33)
                    revert(0,0x20)
                }
            }
        } else if (sysCallCapType == 0x07) {
            // This is a store system call
            // Here we have established that we are processing a write call and
            // we must destructure the necessary values.
            capIndex = parse32ByteValue(1);
            uint256 writeAddress = parse32ByteValue(1+32*1);
            uint256 writeValue = parse32ByteValue(1+32*2);
            cap = procedures.checkWriteCapability(uint192(currentProcedure), writeAddress, capIndex);
            if (cap) {
                assembly {
                    sstore(writeAddress, writeValue)
                    // We don't need to return anything
                    return(0,0)
                }
            } else {
                assembly {
                    mstore(0,22)
                    return(0,0x20)
                }
            }
        } else if (sysCallCapType == 0x09) {
            // This is a log system call
            // Here we have established that we are processing a write call and
            // we must destructure the necessary values.
            capIndex = parse32ByteValue(1);
            uint256 nTopics = parse32ByteValue(1+32*1);
            bytes32[] memory topicVals = new bytes32[](nTopics);
            for (uint256 i = 0; i < nTopics; i++) {
                topicVals[i] = bytes32(parse32ByteValue(1+32*(2+i)));
            }
            bytes32 logValue = bytes32(parse32ByteValue(1+32*(2+nTopics)));
            cap = procedures.checkLogCapability(uint192(currentProcedure), topicVals, capIndex);
            if (cap) {
                if (nTopics == 0) {
                    log0(logValue);
                    assembly {
                        // We don't need to return anything
                        return(0,0)
                    }
                } else if (nTopics == 1) {
                    log1(logValue, topicVals[0]);
                    assembly {
                        // We don't need to return anything
                        return(0,0)
                    }
                } else if (nTopics == 2) {
                    log2(logValue, topicVals[0], topicVals[1]);
                    assembly {
                        // We don't need to return anything
                        return(0,0)
                    }
                } else if (nTopics == 3) {
                    log3(logValue, topicVals[0], topicVals[1], topicVals[2]);
                    assembly {
                        // We don't need to return anything
                        return(0,0)
                    }
                } else if (nTopics == 4) {
                    log4(logValue, topicVals[0], topicVals[1], topicVals[2], topicVals[3]);
                    assembly {
                        // We don't need to return anything
                        return(0,0)
                    }
                } else {
                    assembly {
                        // Revert with an error code
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
        } else if (sysCallCapType == 0x03) {
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

            // TODO: implement, this is issue #58.
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
        // assembly {
        //     sstore(currentProcedure_slot,div(name,exp(0x100,8)))
        // }
        currentProcedure = name;
        address procedureAddress = procedures.get(name);
        bool status = false;
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
                let n := 0
                jumpi(loopend, eq(rsize, 0))
                loop:
                    mstore(add(result,n),0)
                    n := add(n, 32)
                    jumpi(loop, iszero(eq(n, rsize)))
                loopend:
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            // Retrieve the address of new available memory from address 0x40
            let n :=  malloc(0x20)
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
            // There is no return value, therefore it's length is 0 bytes long
            // REVISION: lets assume a 32 byte return value for now
            let outl := 0x20
            let outs := mallocZero(outl)

            status := callcode(gas,procedureAddress,0,ins,inl,outs,outl)
            // Zero-out the currentProcedure
            sstore(currentProcedure_slot,0)
            if eq(status,0) {
                let errStore := malloc(0x20)
                mstore(errStore,add(220000,mload(outs)))
                // mstore(0x40,235)
                return(errStore,0x20)
            }
            if eq(status,1) {
                return(outs,outl)
            }
        }
        if (!status) {
            retVal = 85;
        }
    }
}