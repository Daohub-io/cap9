pragma solidity ^0.4.17;

import "./KernelStorage.sol";

contract ProcedureTable is KernelStorage {

    // CAPABILITY_TYPES
    uint8 constant CAP_PROC_CALL            = 3;
    uint8 constant CAP_PROC_REGISTER        = 4;
    uint8 constant CAP_PROC_DELETE          = 5;
    uint8 constant CAP_PROC_ENTRY           = 6;
    uint8 constant CAP_STORE_WRITE          = 7;
    uint8 constant CAP_LOG                  = 8;
    uint8 constant CAP_ACC_CALL             = 9;

    function _storeProcedure(uint192 key, uint192 keyIndex, address location, uint256[] caps) internal {
        // Procedure List
        // Store the the procedure key in the procedure list
        uint256 keyPointer = _getPointerProcHeapByIndex(keyIndex);
        _set(keyPointer, key);

        // Procedure Heap
        // Get the storage address of the procedure data. This is the storage
        // key which contains all of the procedure data.
        uint256 pPointer = _getPointerProcHeapByName(key);
        _serialiseProcedure(pPointer, keyIndex, location, caps);
    }

    function _serialiseProcedure(uint256 pPointer, uint192 keyIndex, address location, uint256[] caps) internal {
        // Store the address of the code contract
        _set(pPointer + 0, uint256(location));
        // Store the keyIndex
        _set(pPointer + 1, uint256(keyIndex));
        // Clear any previously defined caps by setting the clists to 0-length
        _clearCaps(pPointer);
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
                    thisTypeLength = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | (0)*0x100);
                    if (capIndex >= thisTypeLength) {
                        // The c-list of this type is not long enough.
                        // revert("bad cap index");
                        assembly {
                            mstore(0,0x88)
                            revert(0,32)
                        }
                    }

                    // Serialise the cap.
                    currentLength = _get(pPointer | (capType*0x10000));
                    for (j = 0; (j+3) < capSize; j++) {
                        val = caps[i+3+j];
                        _set(pPointer | (capType*0x10000) | ((currentLength+1)*0x100) | j, val);
                    }
                    // Increment length
                    _set(pPointer | (capType*0x10000), currentLength + 1);


                    val = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | (capIndex + 1)*0x100);
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
                    thisTypeLength = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | (0)*0x100);
                    if (capIndex >= thisTypeLength) {
                        // The c-list of this type is not long enough.
                        // revert("bad cap index");
                        assembly {
                            mstore(0,0x88)
                            revert(0,32)
                        }
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

    // Clear the capabilities of a procedure of the given name.
    function _clearCaps(uint256 pPointer) internal {
        _set(pPointer | (CAP_PROC_CALL*0x10000), 0);
        _set(pPointer | (CAP_PROC_REGISTER*0x10000), 0);
        _set(pPointer | (CAP_PROC_DELETE*0x10000), 0);
        _set(pPointer | (CAP_PROC_ENTRY*0x10000), 0);
        _set(pPointer | (CAP_STORE_WRITE*0x10000), 0);
        _set(pPointer | (CAP_LOG*0x10000), 0);
        _set(pPointer | (CAP_ACC_CALL*0x10000), 0);
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
            currentVal = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | ((capIndex + 1)*0x100) | 0x00);
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
            currentVal = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | 0x00);
            requestedVal = caps[i+3+0];
            if (requestedVal < currentVal) {
                return false;
            }

            // Number of additional storage keys
            currentVal += _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | 0x01);
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
            currentVal = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | 0x00);
            requestedVal = caps[i+3+0];
            if (requestedVal < currentVal) {
                return false;
            }

            // Next we check that the topics required by the current cap are
            // also required by the requested cap.
            for (b = 1; b <= currentVal; b++) {
                if (caps[i+3+b] != _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | ((capIndex+1)*0x100) | b)) {
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
            currentVal = _get(_getPointerProcHeapByName(currentProcedure) | (capType*0x10000) | ((capIndex + 1)*0x100) | 0x00);
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
    function returnRawProcedureTable() public view returns (uint256[]) {
        bytes24[] memory keys = getKeys();
        uint256 len = keys.length;
        // max is 256 keys times the number of procedures
        uint256[] memory r = new uint256[](len*257);
        // The rest are the elements
        uint256 n = 0;
        for (uint256 i = 0; i < len ; i++) {
            uint192 key = uint192(keys[i]);
            uint256 pPointer = _getPointerProcHeapByName(key);
            r[n] = uint256(key); n++;
            for (uint256 j = 0; j < 256; j++) {
                r[n] = _get(pPointer+j); n++;
            }
        }
        return r;
    }

    function returnProcedureTable() public view returns (uint256[]) {
        bytes24[] memory keys = getKeys();
        uint256 len = keys.length;
        // max is 256 keys times the number of procedures
        uint256[] memory r = new uint256[](len*256);
        // The rest are the elements
        uint256 n = 1;
        for (uint256 i = 0; i < len ; i++) {
            // uint192 key = uint192(keys[i]);
            uint256 pPointer = _getPointerProcHeapByName(uint192(keys[i]));
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

    function insert(bytes24 key, address location, uint256[] caps) internal returns (bool inserted) {
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
            uint256 lenP = _getPointerProcedureTableLength();
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

    function remove(bytes24 key) internal returns (bool success) {

        // Get the key index of the procedure we are trying to delete.
        uint192 p1Index = _getProcedureIndex(uint192(key));

        // If the value does not exist in the procedure list we return 0.
        if (p1Index == 0)
            return false;
        // When we remove a procedure, we want to move another procedure into
        // this "slot" to keep the active keys contiguous
        // First we get the storage address of the Procedure Table Length
        uint256 lenP = _getPointerProcedureTableLength();
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
                _set(_getPointerProcedureIndexOnHeap(uint192(key)), 0);
                // 2. Get the key of the last value in the list (we will move
                // that into the newly vacated position)
                uint256 lastKey = _get(procedureIndexToLocation(uint192(len)));
                // 3. Zero out the last value in the procedure list
                _set(procedureIndexToLocation(uint192(len)), 0);
                // 4. Set the newly vacated index to (what was) the last key
                _set(procedureIndexToLocation(p1Index), lastKey);
                // 5. Update the value of the Procedure Index in the procedure
                // heap for this procedure
                _set(_getPointerProcedureIndexOnHeap(uint192(lastKey)), p1Index);
            } else {
                // This procedure is the last in the list, so we simply need to
                // zero out that value, and zero out the Procedure Index in
                // the procedure heap
                // 1. Zero out the Procedure Index value on the heap
                _set(_getPointerProcedureIndexOnHeap(uint192(key)), 0);
                // 2. Zero out the last value in the procedure list
                _set(procedureIndexToLocation(uint192(len)), 0);
            }

            // TODO: this is completely optional so is left for now.
            // Free P1
            // _freeProcedure(p1P);
            uint256 pPointer = _getPointerProcHeapByName(uint192(key));
            _set(pPointer, 0);
            // _set(pPointer + 1, 0);

            return true;
        } else {
            return false;
        }
    }

    function contains(bytes24 key) internal view returns (bool exists) {
        return _get(_getPointerProcHeapByName(uint192(key))) > 0;
    }

    function size() internal view returns (uint) {
        return _get(_getPointerProcedureTableLength());
    }

    function get(bytes24 key) internal view returns (address) {
        return address(_get(_getPointerProcHeapByName(uint192(key)) + 0));
    }

    function procedureIndexToLocation(uint192 procedureIndex) internal pure returns (uint256) {
        uint256 lenP = _getPointerProcedureTableLength();
        return (lenP + (procedureIndex << 24));
    }

    function getKeys() public view returns (bytes24[] memory keys) {
        uint256 lenP = _getPointerProcedureTableLength();
        uint256 len = _get(lenP);
        keys = new bytes24[](len);
        for (uint256 i = 0; i < len; i++) {
            // We use +1 here because the length of the procedure list is
            // stored in the first position
            keys[i] = bytes24(_get(lenP + ((i+1) << 24)));
        }
    }

    function getKeyByIndex(uint8 idx) internal view returns (uint192) {
        return uint192(_get(_getPointerProcHeapByIndex(idx)));
    }

    function getValueByIndex(uint8 idx) internal view returns (address) {
        return address(_get(_getPointerProcHeapByName(getKeyByIndex(idx)) + 1));
    }

}
