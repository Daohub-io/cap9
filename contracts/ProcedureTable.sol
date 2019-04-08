pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;

    struct Procedure {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
        Capability[] caps;
    }

    struct Capability {
        uint8 capType;
        uint256[] values;
    }

    struct Self {}

    // CAPABILITY_TYPES
    uint8 constant CAP_PROC_CALL            = 3;
    uint8 constant CAP_PROC_REGISTER        = 4;
    uint8 constant CAP_PROC_DELETE          = 5;
    uint8 constant CAP_PROC_ENTRY           = 6;
    uint8 constant CAP_STORE_WRITE          = 7;
    uint8 constant CAP_LOG                  = 8;
    uint8 constant CAP_ACC_CALL             = 9;

    function _get(uint256 pointer) internal view returns (uint256 val) {
        assembly {
            // Load Value
            val := sload(pointer)
        }
    }

    function _set(uint256 pointer, uint256 value) internal {
        assembly {
            sstore(pointer, value)
        }
    }

    // Get the procedure currently being executed.
    function _getCurrentProcedure() view internal returns (uint192 val) {
        assembly {
            val := sload(0xffffff0300000000000000000000000000000000000000000000000000000000)
        }
        return val;
    }

    // Return the storage key that holds the number of procedures in the list.
    function _getLengthPointer() internal pure returns (uint256) {
        bytes5 directory = bytes5(0xffffffff01);
        return uint256(bytes32(directory));
    }

    // Returns the storage key that holds the procedure data of procedure #idx
    // in the procedure list. idx starts at 1.
    function _getKeyPointerByIndex(uint192 idx) internal pure returns (uint256) {
        // TODO: annoying error condition, can we avoid it?
        if (idx == 0) {
            revert("0 is not a valid key index");
        }
        bytes5 directory = bytes5(0xffffffff01);
        return uint256(bytes32(directory)) | (uint256(idx) << 24);
    }

    // Returns the storage key that holds the procedure data named by key.
    function _getProcedurePointerByKey(uint192 key) internal pure returns (uint256) {
        bytes5 directory = bytes5(0xffffffff00);
        return uint256(bytes32(directory)) | (uint256(key) << 24);
    }

    // Given a Procedure Key, return it's index in the Procedure List (i.e. its
    // Procedure Index). If the procedure is not in the list it will return a
    // Procedure Index of zero. Zero is not a valid Procedure Index.
    function _getProcedureIndex(uint192 key) internal view returns (uint192) {
        uint256 procedureIndexPointer = _getProcedureIndexPointer(key);
        return uint192(_get(procedureIndexPointer));
    }

    // The storage key that holds the Procedure Index of a procedure with the
    // given key.
    function _getProcedureIndexPointer(uint192 key) internal pure returns (uint256) {
        uint256 pPointer = _getProcedurePointerByKey(key);
        // The procedure index is stored at position 1
        return (pPointer+1);
    }

    // The storage key that holds the Ethereum Address of the code of a
    // procedure with the given key.
    function _getProcedureAddressPointer(uint192 key) internal pure returns (uint256) {
        uint256 pPointer = _getProcedurePointerByKey(key);
        // The procedure index is stored at position 1
        return (pPointer+1);
    }

    function _getProcedureByKey(uint192 key) internal view returns (Procedure memory p) {
        // pPointer is a storage key which points to the start of the procedure
        // data on the procedure heap
        uint256 pPointer = _getProcedurePointerByKey(key);
        // The first storage location (0) is used to store the keyIndex.
        p.keyIndex = uint8(_get(pPointer));
        // The second storage location (1) is used to store the address of the
        // contract.
        p.location = address(_get(pPointer + 1));
        // For now let's just get the number of Procedure Call caps
        uint256 nCallCaps = _get(pPointer | 0x030000);

        // The third storage location (2) is used to store the number of caps
        // uint256 nCaps = _get(pPointer + 2);
        p.caps = new Capability[](nCallCaps);
        // n is the cap index
        uint256 n = 0;
        // The rest of the 256 keys are (or can be) used for the caps
        for (uint256 i = 0; i < (256-3); i++) {
            if (n >= nCallCaps) {
                break;
            }
            p.caps[n].capType = uint8(0x03);
            // A call cap will always have length 1
            uint256 nValues = 1;
            p.caps[n].values = new uint256[](nValues);
            for (uint256 k = 0; k < nValues; k++) {
                p.caps[n].values[k] = uint256(_get((pPointer | 0x030000) + i*(0x100) + k));
            }
            // uint256 thisCurrentCapLength = _get(pPointer+3+i);
            // p.caps[n].capType = uint8(_get(pPointer+3+i+1));
            // // subtract 1 from cap length because it includes the type
            // uint256 nValues = thisCurrentCapLength - 1;
            // // uint256 nValues = 2;
            // p.caps[n].values = new uint256[](nValues);
            // for (uint256 k = 0; k < nValues; k++) {
            //     p.caps[n].values[k] = uint256(_get(pPointer+3+i+2+k));
            // }
            // i = i + uint256(thisCurrentCapLength);
            // n++;
        }
    }

    function checkRegisterCapability(Self storage /* self */, uint192 currentProcedure, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {

        uint256 capType = CAP_PROC_REGISTER;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function checkDeleteCapability(Self storage /* self */, uint192 currentProcedure, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {

        uint256 capType = CAP_PROC_DELETE;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function checkSetEntryCapability(Self storage /* self */, uint192 currentProcedure, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_PROC_ENTRY;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function checkCallCapability(Self storage /* self */, uint192 currentProcedure, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {

        uint256 capType = CAP_PROC_CALL;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function checkAccCallCapability(Self storage /* self */, uint192 currentProcedure, address account, uint256 amount, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_ACC_CALL;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function checkWriteCapability(Self storage /* self */, uint192 currentProcedure, uint256 toStoreAddress, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_STORE_WRITE;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function checkLogCapability(Self storage /* self */, uint192 currentProcedure, bytes32[] reqTopics, uint256 reqCapIndex) internal view returns (bool) {
        uint256 capType = CAP_LOG;
        // Storage key of the current procedure on the procedure heap
        uint256 currentProcPointer = _getProcedurePointerByKey(currentProcedure);
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

    function _storeProcedure(uint192 key, uint192 keyIndex, address location, uint256[] caps) internal {
        // Procedure List
        // Store the the procedure key in the procedure list
        uint256 keyPointer = _getKeyPointerByIndex(keyIndex);
        _set(keyPointer, key);

        // Procedure Heap
        // Get the storage address of the procedure data. This is the storage
        // key which contains all of the procedure data.
        uint256 pPointer = _getProcedurePointerByKey(key);
        _serialiseProcedure(pPointer, keyIndex, location, caps);
    }

    function _serialiseProcedure(uint256 pPointer, uint192 keyIndex, address location, uint256[] caps) internal {
        // Store the address of the code contract
        _set(pPointer + 0, uint256(location));
        // Store the keyIndex
        _set(pPointer + 1, uint256(keyIndex));
        _serialiseCapArray(pPointer, caps);
    }

    function _serialiseCapArray(uint256 pPointer, uint256[] caps) internal {
        uint192 currentProcedure = _getCurrentProcedure();
        // If there is no current procedure, we can do anything.
        // TODO: this is something for consideration, we often ask the kernel to
        // register things directly.

        // Variables for later use
        uint256 currentLength;
        uint256 val;
        uint256 j;
        uint256 thisTypeLength;
        // i is the index into the caps array, which is a series of 32-byte values
        for (uint256 i = 0; (i+2) < caps.length; ) {
            uint256 capSize = caps[i+0];
            uint256 capType = caps[i+1];
            // TODO: check for overflows
            uint8 capIndex = uint8(caps[i+2]);

            if (capSize == 3) {
                // If the capSize is 3, we don't need to create a subset, we just
                // copy whatever is at capIndex

                // We can't deal with this properly yet, but for the pursposes of setEntry, we will just set an empty value
                currentLength = _get(pPointer | (capType*0x10000));
                // Increment length
                _set(pPointer | (capType*0x10000), currentLength + 1);

                if (currentProcedure == 0) {
                    // If there is no currentProcedure we are under direct control
                    // of the kernel, and any capability is the max cap of that type
                    // serialise Procedure Call cap

                    currentLength = _get(pPointer | (capType*0x10000));
                    // We branch here as we need to insert the maximum capability, which varies for each type
                    if (capType == CAP_PROC_CALL) {
                        // Insert a 0 value (which means the prefix will be zero, i.e. the maximum capability)
                        val  = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                    } else if (capType == CAP_STORE_WRITE) {
                        // Insert a base address of zero
                        val = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                        // Insert "number of additional keys as MAX-1
                        val = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x01, val);
                    } else if (capType == CAP_LOG) {
                         // Number of requred caps is zero
                        val = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                        // set topic 1 to zero (TODO: not required but defensive)
                        val = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x01, val);
                        // set topic 2 to zero (TODO: not required but defensive)
                        val = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x02, val);
                        // set topic 3 to zero (TODO: not required but defensive)
                        val = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x03, val);
                        // set topic 4 to zero (TODO: not required but defensive)
                        val = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x04, val);
                    } else if (capType == CAP_PROC_REGISTER) {
                        // Insert a 0 value (which means the prefix will be zero, i.e. the maximum capability)
                        val  = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                    } else if (capType == CAP_PROC_DELETE) {
                        // Insert a 0 value (which means the prefix will be zero, i.e. the maximum capability)
                        val  = 0;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                    } else if (capType == CAP_PROC_ENTRY) {
                        // This cap does not require any values to be set
                    } else if (capType == CAP_ACC_CALL) {
                        // Insert the max value, which is the first 2 bits set on, and the remainder zero
                        val = 0xc000000000000000000000000000000000000000000000000000000000000000;
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                    } else {
                        revert("unknown capability");
                    }
                    // Increment length
                    _set(pPointer | (capType*0x10000), currentLength + 1);
                } else {
                    // Otherwise we need to copy the capability from the current
                    // procedure.

                    // First check that capIndex (from which we derive our cap)
                    // actually exists. This just checks that the list of that
                    // type is long enough.
                    thisTypeLength = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | (0)*0x100);
                    if (capIndex >= thisTypeLength) {
                        // The c-list of this type is not long enough.
                        revert("bad cap index");
                    }

                    // Serialise the cap.
                    currentLength = _get(pPointer | (capType*0x10000));
                    for (j = 0; (j+3) < capSize; j++) {
                        val = caps[i+3+j];
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | j, val);
                    }
                    // Increment length
                    _set(pPointer | (capType*0x10000), currentLength + 1);


                    val = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | (capIndex + 1)*0x100);
                    // Store that in the procedure we are registering
                    // Get the current number of caps of this type for this
                    // proc.
                    currentLength = _get(pPointer | (capType*0x10000));
                    // A Procedure Call cap has one 32-byte value. Set it here at
                    // position 0
                    _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | 0x00, val);
                    // Increment length
                    _set(pPointer | (capType*0x10000), currentLength + 1);
                }
            } else {
                // If the capSize is not three, we want to create a subset.
                // For testing purposes at this stage we will ignore subsets
                // and simply allow whatever is dictated by the request.
                if (currentProcedure == 0) {
                    // Get the current number of caps of this type for this
                    // proc.
                    currentLength = _get(pPointer | (capType*0x10000));
                    for (j = 0; (j+3) < capSize; j++) {
                        val = caps[i+3+j];
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | j, val);
                    }
                    // Increment length
                    _set(pPointer | (capType*0x10000), currentLength + 1);
                } else {
                    // First check that capIndex (from which we derive our cap)
                    // actually exists. This just checks that the list of that
                    // type is long enough.
                    thisTypeLength = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | (0)*0x100);
                    if (capIndex >= thisTypeLength) {
                        // The c-list of this type is not long enough.
                        revert("bad cap index");
                    }

                    bool subset = isSubset(currentProcedure, capType, capIndex, caps, i);
                    if (!subset) {
                        revert("cap not valid subset");
                    }

                    // Serialise the cap.
                    currentLength = _get(pPointer | (capType*0x10000));
                    for (j = 0; (j+3) < capSize; j++) {
                        val = caps[i+3+j];
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | j, val);
                    }
                    // Increment length
                    _set(pPointer | (capType*0x10000), currentLength + 1);
                }
            }
            i += capSize;
        }
    }

    function isSubset(uint192 currentProcedure, uint256 capType, uint256 capIndex, uint256[] caps, uint256 i) public view returns (bool) {
        uint256 currentVal;
        uint256 requestedVal;
        uint256 b;
        uint256 current;
        uint256 req;
        // Check if our cap is a subset. If not revert.
        // The subset logic of these three caps are the same
        if (capType == CAP_PROC_CALL || capType == CAP_PROC_REGISTER || capType == CAP_PROC_DELETE) {
            currentVal = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | ((capIndex + 1)*0x100) | 0x00);
            requestedVal = caps[i+3+0];

            // Check that the prefix of B is >= than the prefix of A.
            current = currentVal & 0xff00000000000000000000000000000000000000000000000000000000000000;
            req = requestedVal & 0xff00000000000000000000000000000000000000000000000000000000000000;
            if (current > req) {
                return false;
            }

            // b is "currentMask"
            b = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff << ((192 - (current >> 248)));
            current = b & currentVal & 0x0000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff;
            req = b & requestedVal & 0x0000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff;
            // Insert a 0 value (which means the prefix will be zero, i.e. the maximum capability)
            // Check that the first $prefix bits of the two keys are the same
            if (current != req) {
                return false;
            }
            return true;
        } else if (capType == CAP_STORE_WRITE) {
            // Base storage address
            currentVal = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | 0x00);
            requestedVal = caps[i+3+0];
            if (requestedVal < currentVal) {
                return false;
            }

            // Number of additional storage keys
            currentVal += _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | 0x01);
            requestedVal += caps[i+3+1];
            if (requestedVal > currentVal) {
                return false;
            }
            // Even though there exists invalid capabilities, we don't check for
            // them here as it wouldn't cover all circumstances. If we wan to
            // check for it we should do it more generally.
            // if (requestedVal == 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff) {
            //     return false;
            // }
            return true;
        } else if (capType == CAP_LOG) {
            // First we check the number of required topics. The number of
            // required topics of the requested cap must be equal to or greater
            // than the number of required topics for the current cap.
            currentVal = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | 0x00);
            requestedVal = caps[i+3+0];
            if (requestedVal < currentVal) {
                return false;
            }

            // Next we check that the topics required by the current cap are
            // also required by the requested cap.
            for (b = 1; b <= currentVal; b++) {
                if (caps[i+3+b] != _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | b)) {
                    return false;
                }
            }
            return true;
        } else if (capType == CAP_PROC_ENTRY) {
            // All of these caps are identical, therefore any cap of this type
            // is the subset of another
            return true;
        } else if (capType == CAP_ACC_CALL) {
            // If the requested value of callAny is true, then the requested
            // value of callAny must be true.
            currentVal = _get(_getProcedurePointerByKey(currentProcedure) | (capType*0x10000) | ((capIndex + 1)*0x100) | 0x00);
            requestedVal = caps[i+3+0];
            current = currentVal & 0x8000000000000000000000000000000000000000000000000000000000000000;
            req = requestedVal & 0x8000000000000000000000000000000000000000000000000000000000000000;
            // If req != 0 (that is, equals 1, requested callAny flag is true) and
            // current == 0 (that is, current callAny flag is false)
            // then fail
            if (req != 0) {
                // requested callAny == true
                if (current == 0) {
                    return false;
                }
            } else {
                // requested callAny == false
                // if the current value is callAny, we don't care about the
                // value of ethAddress. If the current value of callAny is
                // 0 (false) we must check that the addresses are the same
                if (current == 0) {
                    // the addresses must match
                    // get the current and required addresses
                    current = currentVal & 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff;
                    req = requestedVal & 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff;
                    if (current != req) {
                        return false;
                    }
                }
            }
            // if the requested sendValue flag is true, the current sendValue
            // flag must also be true.
            // get the sendValue flags
            current = currentVal & 0x4000000000000000000000000000000000000000000000000000000000000000;
            req = requestedVal & 0x4000000000000000000000000000000000000000000000000000000000000000;
            if (req != 0 && current == 0) {
                return false;
            }
            return true;
        } else {
            revert("unknown capability");
        }
    }

    // Just returns an array of all the procedure data (257 32-byte values) concatenated.
    function returnRawProcedureTable(Self storage self) internal view returns (uint256[]) {
        bytes24[] memory keys = self.getKeys();
        uint256 len = keys.length;
        // max is 256 keys times the number of procedures
        uint256[] memory r = new uint256[](len*257);
        // The rest are the elements
        uint256 n = 0;
        for (uint256 i = 0; i < len ; i++) {
            uint192 key = uint192(keys[i]);
            uint256 pPointer = _getProcedurePointerByKey(key);
            r[n] = uint256(key); n++;
            for (uint256 j = 0; j < 256; j++) {
                r[n] = _get(pPointer+j); n++;
            }
        }
        return r;
    }

    function returnProcedureTable(Self storage self) internal view returns (uint256[]) {
        bytes24[] memory keys = self.getKeys();
        uint256 len = keys.length;
        // max is 256 keys times the number of procedures
        uint256[] memory r = new uint256[](len*256);
        // The rest are the elements
        uint256 n = 1;
        for (uint256 i = 0; i < len ; i++) {
            // uint192 key = uint192(keys[i]);
            uint256 pPointer = _getProcedurePointerByKey(uint192(keys[i]));
            r[n] = uint192(keys[i]); n++;
            // Store the keyIndex at this location
            r[n] = _get(pPointer); n++;
            r[n] = _get(pPointer + 1); n++;
            // Save this spot to record the the total number of caps
            uint256 nCapTypesLocation = n; n++;
            uint256 totalCaps = 0;
            // Cycle through all cap types [0,255], even though most don't exist
            for (uint256 j = 1; j <= 10; j++) {
                // How many caps are there of this type
                uint256 nCaps = _get(pPointer | (j*0x10000) | 0x00 | 0x00);
                // Only record the caps if they're at least 1W
                if (nCaps > 0) {
                    uint256 capSize = capTypeToSize(j);
                    // Cycle through them and add them to the array. Here we need to
                    // know the size.
                    for (uint256 k = 1; k <= nCaps; k++) {
                    // record the size
                    r[n] = (capSize+2); n++;
                    // record the type
                    r[n] = j; n++;
                        totalCaps++;
                        for (uint256 l = 0; l < capSize; l++) {
                            r[n] = _get(pPointer | (j*0x10000) | (k*0x100) | (l*0x1)); n++;
                        }
                    }
                }
            }
            r[nCapTypesLocation] = totalCaps;
        }
        r[0] = n;
        return r;
    }

    function capTypeToSize(uint256 capType) internal pure returns (uint256) {
        if (capType == CAP_PROC_CALL) {
            return 1;
        } else if (capType == CAP_STORE_WRITE) {
            return 2;
        } else if (capType == CAP_LOG) {
            return 5;
        } else if (capType == CAP_PROC_REGISTER) {
            return 1;
        } else if (capType == CAP_PROC_DELETE) {
            return 1;
        } else if (capType == CAP_PROC_ENTRY) {
            return 0;
        } else if (capType == CAP_ACC_CALL) {
            return 1;
        } else {
            revert("invalid capability type");
        }

    }

    function insert(Self storage /* self */, bytes24 key, address location, uint256[] caps) internal returns (bool inserted) {
        // First we get retrieve the procedure that is specified by this key, if
        // it exists, otherwise the struct we create in memory is just
        // zero-filled.
        uint192 keyIndex = _getProcedureIndex(uint192(key));
        // we just copy in the table in verbatim as long as its length is less
        // than 128 (arbitrary, but less than 256 minus room for other parameters)
        if (caps.length > 128) {
            revert();
        }

        // If the keyIndex is not zero then that indicates that the procedure
        // already exists. In this case *WE HAVE NOT OVERWRITTEN * the values,
        // as we have not called _storeProcdure.
        if (keyIndex > 0) {
            return false;
        // If the keyIndex is zero (it is unsigned and cannot be negative) then
        // it means the procedure is new. We must therefore assign it a key
        // index.
        } else {
            // First we retrieve a pointer to the Procedure Table Length value.
            uint256 lenP = _getLengthPointer();
            // We then dereference that value.
            uint256 len = _get(lenP);
            // We assign this procedure the next keyIndex, i.e. len+1
            keyIndex = uint8(len + 1);
            // We increment the Procedure Table Length value
            _set(lenP, len + 1);
            // We actually commit the values in p to storage
            _storeProcedure(uint192(key), keyIndex, location, caps);
            return true;
        }
    }

    function _parseCaps(Procedure memory p, uint256[] caps) internal pure {
        // count caps
        uint256 nCaps = 0;
        for (uint256 i = 0; i < caps.length; i++) {
            uint256 capLength = caps[i];
            i = i + capLength;
            nCaps++;
        }
        p.caps = new Capability[](nCaps);
        uint256 n = 0;
        for (i = 0; i < caps.length; ) {
            capLength = caps[i]; i++;
            uint256 nValues = capLength - 1;
            p.caps[n].values = new uint256[](nValues);
            p.caps[n].capType = uint8(caps[i]); i++;
            for (uint256 j = 0; j < nValues; j++) {
                p.caps[n].values[j] = caps[i];i++;
            }
            n++;
        }
    }

    function remove(Self storage /* self */, bytes24 key) internal returns (bool success) {

        // Get the key index of the procedure we are trying to delete.
        uint192 p1Index = _getProcedureIndex(uint192(key));

        // If the value does not exist in the procedure list we return 0.
        if (p1Index == 0)
            return false;
        // When we remove a procedure, we want to move another procedure into
        // this "slot" to keep the active keys contiguous
        // First we get the storage address of the Procedure Table Length
        uint256 lenP = _getLengthPointer();
        // We then dereference that to get the value
        uint256 len = _get(lenP);

        if (p1Index <= len) {
            // Three things need to happen:
            //   When a procedure is deleted from the kernel:
            // 1. If the procedure key is the same as the Entry Procedure Key, abort and
            // throw an error.
            // 2. If the procedure key does not exist in the list (i.e. when looking on the
            // procedure heap no procedure index is associated with it), abort and throw
            // an error.
            // 3. The length value is decremented by one.
            // 4. If the procedure being deleted is not the last in the list (i.e. it’s procedure
            // index does not equal the length of the procedure list), the last in the list is
            // copied to overwrite the key being deleted. This also accounts for the case
            // of an empty list.

            // Decrement Keys Length
            _set(lenP, len - 1);

            if (p1Index != len) {
                // 1. Zero out the Procedure Index value on the heap
                _set(_getProcedureIndexPointer(uint192(key)), 0);
                // 2. Get the key of the last value in the list (we will move
                // that into the newly vacated position)
                uint256 lastKey = _get(procedureIndexToLocation(uint192(len)));
                // 3. Zero out the last value in the procedure list
                _set(procedureIndexToLocation(uint192(len)), 0);
                // 4. Set the newly vacated index to (what was) the last key
                _set(procedureIndexToLocation(p1Index), lastKey);
                // 5. Update the value of the Procedure Index in the procedure
                // heap for this procedure
                _set(_getProcedureIndexPointer(uint192(lastKey)), p1Index);
            } else {
                // This procedure is the last in the list, so we simply need to
                // zero out that value, and zero out the Procedure Index in
                // the procedure heap
                // 1. Zero out the Procedure Index value on the heap
                _set(_getProcedureIndexPointer(uint192(key)), 0);
                // 2. Zero out the last value in the procedure list
                _set(procedureIndexToLocation(uint192(len)), 0);
            }

            // TODO: this is completely optional so is left for now.
            // Free P1
            // _freeProcedure(p1P);
            uint256 pPointer = _getProcedurePointerByKey(uint192(key));
            _set(pPointer, 0);
            // _set(pPointer + 1, 0);

            return true;
        } else {
            return false;
        }
    }

    function contains(Self storage /* self */, bytes24 key) internal view returns (bool exists) {
        return _get(_getProcedurePointerByKey(uint192(key))) > 0;
    }

    function size(Self storage /* self */) internal view returns (uint) {
        return _get(_getLengthPointer());
    }

    function get(Self storage /* self */, bytes24 key) internal view returns (address) {
        return address(_get(_getProcedurePointerByKey(uint192(key)) + 0));
    }

    function procedureIndexToLocation(uint192 procedureIndex) internal pure returns (uint256) {
        uint256 lenP = _getLengthPointer();
        return (lenP + (procedureIndex << 24));
    }

    function getKeys(Self storage /* self */) internal view returns (bytes24[] memory keys) {
        uint256 lenP = _getLengthPointer();
        uint256 len = _get(lenP);
        keys = new bytes24[](len);
        for (uint256 i = 0; i < len; i++) {
            // We use +1 here because the length of the procedure list is
            // stored in the first position
            keys[i] = bytes24(_get(lenP + ((i+1) << 24)));
        }
    }

    function getKeyByIndex(Self storage /* self */, uint8 idx) internal view returns (uint192) {
        return uint192(_get(_getKeyPointerByIndex(idx)));
    }

    function getValueByIndex(Self storage self, uint8 idx) internal view returns (address) {
        return address(_get(_getProcedurePointerByKey(self.getKeyByIndex(idx)) + 1));
    }

}
