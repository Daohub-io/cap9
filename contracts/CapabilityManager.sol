pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract CapabilityManager is ProcedureTable {

    function checkRegisterCapability(uint192 currentProcedure, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {

        uint256 capType = CAP_PROC_REGISTER;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Procedure Call capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability;
        if (reqCapIndex+1 > nCaps) {
            return false;
        }
        // A procedure call capabilities stores a single 32-byte value at 0x00.
        uint256 value = _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x00);
        // This value has to be destructured to get 2 values, a prefix length
        // and a base address.
        uint8 prefix;
        bytes24 baseKey;
        bytes24 clearedBaseKey;
        bytes24 clearedReqKey;
        assembly {
            // Shift the 32-byte value to the right to obtain the first byte only
            prefix := div(value,0x100000000000000000000000000000000000000000000000000000000000000)
            // Shift the value to get the procedure key left align (as it should
            // be for compatibility with bytes24).
            baseKey := mul(value,0x10000000000000000)
            let q := signextend(1,2)
            // h is a large number we will use for arithmetic
            let h := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            // y is a number with $prefix 1s at the start
            let y := mul(h,exp(2,sub(256,prefix)))
            clearedBaseKey := and(y,baseKey)
            clearedReqKey := and(y,procedureKey)
        }
        return clearedBaseKey == clearedReqKey;
    }

    function checkDeleteCapability(uint192 currentProcedure, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {

        uint256 capType = CAP_PROC_DELETE;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Procedure Call capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability;
        if (reqCapIndex+1 > nCaps) {
            return false;
        }
        // A procedure delete capability stores a single 32-byte value at 0x00.
        uint256 value = _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x00);
        // This value has to be destructured to get 2 values, a prefix length
        // and a base address.
        uint8 prefix;
        bytes24 baseKey;
        bytes24 clearedBaseKey;
        bytes24 clearedReqKey;
        assembly {
            // Shift the 32-byte value to the right to obtain the first byte only
            prefix := div(value,0x100000000000000000000000000000000000000000000000000000000000000)
            // Shift the value to get the procedure key left align (as it should
            // be for compatibility with bytes24).
            baseKey := mul(value,0x10000000000000000)
            let q := signextend(1,2)
            // h is a large number we will use for arithmetic
            let h := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            // y is a number with $prefix 1s at the start
            let y := mul(h,exp(2,sub(256,prefix)))
            clearedBaseKey := and(y,baseKey)
            clearedReqKey := and(y,procedureKey)
        }
        return clearedBaseKey == clearedReqKey;
    }

    function checkSetEntryCapability(uint192 currentProcedure, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_PROC_ENTRY;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Write capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability.
        // NB: Even though all set entry caps are identical, if you ask for an
        // index that doesn't exist, we will still return false. This implies
        // that you should always ask for the cap at index zero to be on the
        // safe side.
        if (reqCapIndex+1 > nCaps) {
            return false;
        } else {
            return true;
        }
        // A set entry capability has no values
    }

    function checkCallCapability(uint192 currentProcedure, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {

        uint256 capType = CAP_PROC_CALL;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Procedure Call capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability;
        if (reqCapIndex+1 > nCaps) {
            return false;
        }
        // A procedure call capabilities stores a single 32-byte value at 0x00.
        uint256 value = _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x00);
        // This value has to be destructured to get 2 values, a prefix length
        // and a base address.
        uint8 prefix;
        bytes24 baseKey;
        bytes24 clearedBaseKey;
        bytes24 clearedReqKey;
        assembly {
            // Shift the 32-byte value to the right to obtain the first byte only
            prefix := div(value,0x100000000000000000000000000000000000000000000000000000000000000)
            // Shift the value to get the procedure key left align (as it should
            // be for compatibility with bytes24).
            baseKey := mul(value,0x10000000000000000)
            let q := signextend(1,2)
            // h is a large number we will use for arithmetic
            let h := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            // y is a number with $prefix 1s at the start
            let y := mul(h,exp(2,sub(256,prefix)))
            clearedBaseKey := and(y,baseKey)
            clearedReqKey := and(y,procedureKey)
        }
        return clearedBaseKey == clearedReqKey;
    }

    function checkAccCallCapability(uint192 currentProcedure, address account, uint256 amount, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_ACC_CALL;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Write capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability;
        if (reqCapIndex+1 > nCaps) {
            return false;
        }
        // A write capability has 2-3 values, callAny: Boolean, sendValue: Boolean,
        // and ethAddress: EthereumAddress. ethAddress is only defined if
        // callAny is false. These values are packed into a single 32-byte value.
        uint256 value = _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x00);
        // This value has to be destructured to get 2 values, a prefix length
        // and a base address.
        // The two flags, callAny and sendValue are stored in the first byte
        // (which we will call the flagByte).
        uint8 flagByte;
        assembly {
            flagByte := byte(0,value)
        }

        // Select the first bit
        // Solidity does not allow conversion of ints to bools, so we do it
        // explicitly.
        bool callAny;
        if ((flagByte & 0x80) > 0) { // 0x80 == 0b100000000;
            callAny = true;
        }
        // Select the second bit
        // Solidity does not allow conversion of ints to bools, so we do it
        // explicitly.
        bool sendValue;
        if ((flagByte & 0x40) > 0) { // 0x40 == 0b010000000;
            sendValue = true;
        }

        // We probably don't need to clear these bits, but it is defensive coding.
        address ethAddress = address(value & 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff); // clear all but last 20-bytes

        // If callAny is false (0) and ethAddress does not match the requested
        // account, return false
        if (!callAny && (ethAddress != account)) {
            return false;
        }

        // If sendValue is false (0) and amount is non-zero, return false
        if (!sendValue && (amount != 0)) {
            return false;
        }

        // Otherwise return true
        return true;
    }

    function checkWriteCapability(uint192 currentProcedure, uint256 toStoreAddress, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_STORE_WRITE;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Write capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability;
        if (reqCapIndex+1 > nCaps) {
            return false;
        }
        // A write capability has two values, address and size. Address is at
        // 0x00 and size is at 0x01.
        uint256 addr = _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x00);
        uint256 size = _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x01);

        // If the store addess is within the range return true, else false
        return (toStoreAddress >= addr && toStoreAddress <= (addr + size));
    }

    function checkLogCapability(uint192 currentProcedure, bytes32[] reqTopics, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_LOG;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getPointerProcHeapByName(currentProcedure);
        // How many Write capabilities does the current procedure have?
        uint256 nCaps = _get(currentProcPointer | (capType*0x10000));
        // If the requested cap is out of the bounds of the cap list, we
        // clearly don't have the capability;
        if (reqCapIndex+1 > nCaps) {
            return false;
        }
        // A log capability has 5 values. The first is the number of topics
        // specified and must be in the range [0,4]. The next 4 values are the
        // values that those log topics are required to be.

        uint256 nTopics =         _get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x00);
        bytes32 topic1  = bytes32(_get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x01));
        bytes32 topic2  = bytes32(_get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x02));
        bytes32 topic3  = bytes32(_get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x03));
        bytes32 topic4  = bytes32(_get(currentProcPointer | (capType*0x10000) | (reqCapIndex + 1)*0x100 | 0x04));

        // Check that all of the topics required by the cap are satisfied. That
        // is, for every topic in the capability, the corresponding exists in
        // the system call and is set to that exact value. First we check that
        // there are enough topics in the request.
        if (reqTopics.length < nTopics) {
            // The system call specifies an insufficient number of topics
            return false;
        }

        if (nTopics >= 1) {
            if (reqTopics[0] != topic1) {
                return false;
            }
        }
        if (nTopics >= 2) {
            if (reqTopics[1] != topic2) {
                return false;
            }
        }
        if (nTopics >= 3) {
            if (reqTopics[2] != topic3) {
                return false;
            }
        }
        if (nTopics >= 4) {
            if (reqTopics[3] != topic4) {
                return false;
            }
        }
        return true;
    }
}
