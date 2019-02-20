pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

library WhatIsMyAddress {
    function get() public view returns (address) {
        return msg.sender;
    }
}

// Public Kernel Interface
contract IKernel {
    event KernelLog(string message);

    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    // Current Entry Procedure
    bytes24 public entryProcedure;

    // SYSCALL_RESPONSE_TYPES
    uint8 constant SyscallSuccess = 0;
    uint8 constant SyscallReadError = 11;
    uint8 constant SyscallWriteError = 22;
    uint8 constant SyscallLogError = 33;
    uint8 constant SyscallCallError = 44;

    uint8 constant NoSuchSyscallError = 111;

    // CAPABILITY_TYPES
    uint8 constant CAP_NULL                 = 0;
    uint8 constant CAP_PROC_CAP_PUSH        = 1;
    uint8 constant CAP_PROC_CAP_DELETE      = 2;
    uint8 constant CAP_PROC_CALL            = 3;
    uint8 constant CAP_PROC_REGISTER        = 4;
    uint8 constant CAP_PROC_DELETE          = 5;
    uint8 constant CAP_PROC_ENTRY           = 6;
    uint8 constant CAP_STORE_WRITE          = 7;
    uint8 constant CAP_LOG                  = 8;
    uint8 constant CAP_GAS_SEND             = 9;

    function getEntryProcedure() public view returns (bytes24 key) {
        return entryProcedure;
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

    function getProcedure(bytes24 name) public view returns (address) {
        return procedures.get(name);
    }

}

// Internal Kernel Interface
contract Kernel is Factory, IKernel {

    // Current Instance Address
    address kernelAddress;
    // Current Running Procedure
    bytes24 currentProcedure;

    constructor() public {}

    function parse32ByteValue(uint256 startOffset) pure internal returns (uint256) {
        uint256 value = 0;
        for (uint256 i = 0; i < 32; i++) {
            value = value << 8;
            value = value | uint256(msg.data[startOffset+i]);
        }
        return value;
    }

    function parse24ByteValue(uint256 startOffset) pure internal returns (uint192) {
        uint192 value = 0;
        for (uint192 i = 0; i < 24; i++) {
            value = value << 8;
            value = value | uint192(msg.data[startOffset+i]);
        }
        return value;
    }

    function parse20ByteValue(uint256 startOffset) pure internal returns (uint160) {
        uint160 value = 0;
        for (uint160 i = 0; i < 20; i++) {
            value = value << 8;
            value = value | uint160(msg.data[startOffset+i]);
        }
        return value;
    }


    // Check if a transaction is external.
    function _isExternal() internal view returns (bool) {
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
    function _callGuardProcedure(address /* sender */, bytes /* data */) internal {
        // TODO: we need to pass through the sender somehow
        // _executeProcedure will call RETURN or REVERT, ending the transaction,
        // so control should never return here
        _executeProcedure(entryProcedure, "", msg.data);
    }

    // This is the fallback function which is used to handle system calls. This
    // is only called if the other functions fail.
    function() public {
        // This is the entry point for the kernel

        // If it is an external account, we forward it straight to the init
        // procedure. We currently shouldn't reach this point, as we usually use
        // "_executeProcedure"
        if (_isExternal()) {
            _callGuardProcedure(msg.sender, msg.data);
        }

        // Parse the system call
        uint8 sysCallCapType = uint8(msg.data[0]);

        // Here we decode the system call (if there is one)
        if (sysCallCapType == 0) {
            // non-syscall case
        } else if (sysCallCapType == CAP_PROC_CALL) {
            _callSystemCall();
        } else if (sysCallCapType == CAP_STORE_WRITE) {
            _storeSystemCall();
        } else if (sysCallCapType == CAP_LOG) {
            _logSystemCall();
        } else if (sysCallCapType == CAP_PROC_REGISTER) {
            _procRegSystemCall();
        } else if (sysCallCapType == CAP_PROC_DELETE) {
            _procDelSystemCall();
        } else if (sysCallCapType == CAP_PROC_ENTRY) {
            _setEntrySystemCall();
        } else {
            // default; fallthrough action
            assembly {
                mstore8(0xb0,5)
                log0(0xb0, 1)
                mstore8(0xb0,5)
                return(0xb0, 1)
            }
        }
    }

    function _callSystemCall() internal {
        // This is a call system call
        // parse a 32-byte value at offset 1 (offset 0 is the capType byte)
        uint256 capIndex = parse32ByteValue(1);
        // parse a 32-byte value at offset 1 (offset 0 is the capType byte,
        // offset 1 is the capIndex (32 bytes)
        // We also perform a shift as this is 24 byte value, not a 32 byte
        // value
        bytes24 procedureKey = bytes24(parse32ByteValue(1+32)/0x10000000000000000);
        uint256 dataLength;
        // log1(bytes32(msg.data.length), bytes32("msg.data.length"));
        if (msg.data.length > (1+3*32)) {
            dataLength = msg.data.length - (1+3*32);
        } else {
            dataLength = 0;
        }
        bool cap = procedures.checkCallCapability(uint192(currentProcedure), procedureKey, capIndex);
        address procedureAddress = procedures.get(procedureKey);
        // Note the procedure we are currently running, we will put this
        // back into the "currentProcedure" after we have finished the call.
        bytes24 previousProcedure = currentProcedure;
        // We set the value for the current procedure in the kernel so that
        // it knows which procedure it is executing (this is important for
        // looking up capabilities).
        currentProcedure = procedureKey;
        // log1(bytes32(procedureKey),bytes32("calling"));
        if (cap) {
            // log1(bytes32("permitted"),bytes32("call-cap"));
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
                }
                if iszero(dataLength) {
                    // If there is not data to be sent we just set the
                    // location and length of the input data to zero. The
                    // location doesn't actually matter as long the length
                    // is zero.
                    ins := 0
                    inl := 0
                }
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
                    // The starting memory offset to place the output data
                    // We are using returndatasize and returndata copy so we set
                    // it to zero
                    0,
                    // The length of the output data
                    // We are using returndatasize and returndata copy so we set
                    // it to zero
                    0)
                let returnLength := returndatasize
                let retLoc := malloc(returnLength)
                returndatacopy(retLoc, 0, returnLength)
                // We need to restore the previous procedure as the current
                // procedure, this can simply be on the stack
                sstore(currentProcedure_slot,div(previousProcedure,exp(0x100,8)))
                if iszero(status) {
                    let errStore := malloc(0x20)
                    mstore(errStore,add(22,mload(retLoc)))
                    revert(errStore,0x20)
                }
                if eq(status,1) {
                    return(retLoc,returnLength)
                }
            }
        } else {
            // log1(bytes32("not-permitted"),bytes32("call-cap"));
            assembly {
                // 33 means the capability was rejected
                mstore(0,33)
                revert(0,0x20)
            }
        }
    }

    function _procRegSystemCall() internal {
        // This is a procedure-register system call
        // this is the system call to register a contract as a procedure
        // currently we enforce no caps

        uint256 capIndex = parse32ByteValue(1);
        // TODO: fix this double name variable work-around
        bytes32 regNameB = bytes32(parse32ByteValue(1+32));
        bytes24 regName = bytes24(regNameB);
        address regProcAddress = address(parse32ByteValue(1+32+32));
        // the general format of a capability is length,type,values, where
        // length includes the type
        uint256 capsStartOffset =
            /* sysCallCapType */ 1
            /* capIndex */ + 32
            /* name */ + 32
            /* address */ + 32;
        // capsLength is the length of the caps arry in bytes
        uint256 capsLengthBytes = msg.data.length - capsStartOffset;
        uint256 capsLengthKeys  = capsLengthBytes/32;
        if (capsLengthBytes % 32 != 0) {
            revert("caps are not aligned to 32 bytes");
        }
        uint256[] memory regCaps = new uint256[](capsLengthKeys);
        for (uint256 q = 0; q < capsLengthKeys; q++) {
            regCaps[q] = parse32ByteValue(capsStartOffset+q*32);
        }
        bool cap = procedures.checkRegisterCapability(uint192(currentProcedure), capIndex);
        if (cap) {

            (uint8 err, /* address addr */) = _registerProcedure(regName, regProcAddress, regCaps);
            uint256 bigErr = uint256(err);
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
                let retSize := 32
                let retLoc := mallocZero(retSize)
                mstore(retLoc,bigErr)
                return(retLoc,retSize)
            }
        } else {
            assembly {
                // 33 means the capability was rejected
                mstore(0,33)
                revert(0,0x20)
            }
        }
    }

    function _procDelSystemCall() internal {
        // This is a procedure-delete system call
        // this is the system call to delete a contract as a procedure
        // currently we enforce no caps

        uint256 capIndex = parse32ByteValue(1);
        // TODO: fix this double name variable work-around
        bytes32 regNameB = bytes32(parse32ByteValue(1+32));
        bytes24 regName = bytes24(regNameB);
        bool cap = procedures.checkDeleteCapability(uint192(currentProcedure), capIndex);
        if (cap) {
            (uint8 err, /* address addr */) = _deleteProcedure(regName);
            uint256 bigErr = uint256(err);
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
                let retSize := 32
                let retLoc := mallocZero(retSize)
                mstore(retLoc,bigErr)
                return(retLoc,retSize)
            }
        } else {
            assembly {
                // 33 means the capability was rejected
                mstore(0,33)
                revert(0,0x20)
            }
        }
    }

    function _setEntrySystemCall() internal {
        // This is a procedure-delete system call
        // this is the system call to delete a contract as a procedure
        // currently we enforce no caps

        uint256 capIndex = parse32ByteValue(1);
        // TODO: fix this double name variable work-around
        bytes32 regNameB = bytes32(parse32ByteValue(1+32));
        bytes24 regName = bytes24(regNameB);
        bool cap = procedures.checkSetEntryCapability(uint192(currentProcedure), capIndex);
        if (cap) {
            (uint8 err) = _setEntryProcedure(regName);
            uint256 bigErr = uint256(err);
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
                let retSize := 32
                let retLoc := mallocZero(retSize)
                mstore(retLoc,bigErr)
                return(retLoc,retSize)
            }
        } else {
            assembly {
                // 33 means the capability was rejected
                mstore(0,33)
                revert(0,0x20)
            }
        }
    }

    function _storeSystemCall() internal {
        // This is a store system call
        // Here we have established that we are processing a write call and
        // we must destructure the necessary values.
        uint256 capIndex = parse32ByteValue(1);
        uint256 writeAddress = parse32ByteValue(1+32*1);
        uint256 writeValue = parse32ByteValue(1+32*2);
        bool cap = procedures.checkWriteCapability(uint192(currentProcedure), writeAddress, capIndex);
        if (cap) {
            assembly {
                sstore(writeAddress, writeValue)
                // We don't need to return anything
                // Return SyscallSuccess = 0
                return(0,0)
            }
        } else {
            assembly {
                // Return SyscallWriteError = 22
                mstore(0, 22)
                return(0,0x20)
            }
        }
    }

    function _logSystemCall() internal {
        // This is a log system call
            // Here we have established that we are processing a write call and
            // we must destructure the necessary values.
            uint256 capIndex = parse32ByteValue(1);
            // this is parsing the number of topics from the system call
            uint256 nTopics = parse32ByteValue(1+32*1);
            bytes32[] memory topicVals = new bytes32[](nTopics);
            for (uint256 i = 0; i < nTopics; i++) {
                topicVals[i] = bytes32(parse32ByteValue(1+32*(2+i)));
            }
            bytes32 logValue = bytes32(parse32ByteValue(1+32*(2+nTopics)));
            bool cap = procedures.checkLogCapability(uint192(currentProcedure), topicVals, capIndex);
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
    }

    // Create a validated procedure.
    function _registerProcedure(bytes24 name, address procedureAddress, uint256[] caps) internal returns (uint8 err, address retAddress) {
        if (validateContract(procedureAddress) == 0) {
            return _registerAnyProcedure(name, procedureAddress, caps);
        } else {
            revert("procedure code failed validation");
        }
    }

    // Create a procedure without  going through any validation. This is mainly
    // used for testing and should not exist in a production kernel.
    function _registerAnyProcedure(bytes24 name, address procedureAddress, uint256[] caps) internal returns (uint8 err, address retAddress) {
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

        procedures.insert(name, procedureAddress, caps);
        retAddress = procedureAddress;
        err = 0;
        return (0, procedureAddress);
    }

    function _setEntryProcedure(bytes24 name) internal returns (uint8 err) {
        // TODO: check that the procedure exists
        entryProcedure = name;
        err = 0;
    }

    function _deleteProcedure(bytes24 name) internal returns (uint8 err, address procedureAddress) {
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


    function _executeProcedure(bytes24 name, string fselector, bytes payload) internal returns (uint256 retVal) {
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
            // Allocate new memory on the stack for function selector and
            // payload data.
            let inl := 0
            // if we have a function selector, we start with a length of 4
            // bytes
            // If there is no function selector, send nothing. mload(fselector)
            // is the length.
            if mload(fselector) {
                // set the input length to 4
                inl := 4
            }
            // Then we add on the length of the payload
            inl := add(inl,mload(payload))

            // n is the actual size we allocate for the input buffer (which may
            // be a little more than we actually need to send)
            let n := inl
            // We need at least 0x20 bytes to perform our hash in, even though
            // we won't send it all
            if lt(n,0x20) {
                n := 0x20
            }
            let ins := malloc(n)

            // next we need to create the function selector hash (if it exists)
            if mload(fselector) {
                // we don't need to allocate memory as we already have enough
                // space in the input buffer (we always allocate at lest 0x20)
                // // allocate some memory to work with
                // let n :=  malloc(0x20)
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
                // This hash must be written first, as it write 32 bytes and
                // would overwrite our payload data. We will store the payload
                // data after this so the unused hash bytes get overwrittem.
                mstore(ins,keccak256(add(fselector,0x20),mload(fselector)))
            }
            // Copy the payload data into the input buffer
            // i starts as the payload length
            let i := mload(payload)
            // The start offset of payload data (either 0 or 4);
            let payloadStart := add(ins,0)
            if mload(fselector) {
                payloadStart := add(ins,4)
            }
            if i {
                for { } i { i := sub(i,  1) } {
                    mstore8(add(payloadStart,sub(i,  1)),mload(add(payload,i)))
                }
            }

            status := callcode(gas,procedureAddress,0,ins,inl,0,0)
            // copy the return data to memory based on its size
            let retSize := returndatasize
            let retLoc := malloc(retSize)
            returndatacopy(retLoc, 0, retSize)
            // Zero-out the currentProcedure
            sstore(currentProcedure_slot,0)
            if eq(status,0) {
                let errStore := malloc(0x20)
                mstore(errStore,add(220000,mload(retLoc)))
                // mstore(0x40,235)
                // log1(errStore,0x20,"returnedErr")
                return(errStore,0x20)
            }
            if eq(status,1) {
                // log1(outs,outl,"returned")
                return(retLoc,retSize)
            }
        }
        if (!status) {
            retVal = 85;
        }
    }

    function _addCap(bytes24 name, uint256[] caps) internal returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }

        bool success = procedures.addCap(name, caps);

        // Check whether the address exists
        if (!success) {err = 2;}
    }
}
