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

    // Convert Pointer To File Pointer
    // Takes a single byte and a full 256 bit storage location
    function _filePointer(uint8 fileId, uint248 pointer) internal pure returns (uint256) {
        // Mask to Uint256
        // Overwrite the most significant byte of pointer with fileId
        return uint256(pointer) | (uint256(fileId) << 248);
    }

    function _get(uint8 fileId, uint248 _pointer) internal view returns (uint256 val) {
        uint256 pointer = _filePointer(fileId, _pointer);
        assembly {
            // Load Value
            val := sload(pointer)
        }
    }

    function _set(uint8 fileId, uint248 _pointer, uint256 value) internal {
        // Convert Mask to Uint256
        uint256 pointer = _filePointer(fileId, _pointer);
        assembly {
            sstore(pointer, value)
        }
    }

    function _getLengthPointer() internal pure returns (uint248) {
        bytes8 directory = bytes8(keccak256("keyPointer"));
        return uint248(directory) << 240;
    }
    function _getKeyPointerByIndex(uint8 idx) internal pure returns (uint248) {
        bytes8 directory = bytes8(keccak256("keyPointer"));
        return (uint248(directory) << 240) + 1 + uint248(idx);
    }

    function _getProcedurePointerByKey(uint192 key) internal pure returns (uint248) {
        // Procedure data is stored under the procedurePointer "directory". The
        // location of the procedure data is followed by the name/key of the
        // procedure.
        // keccak256("procedurePointer") is 0x85a94e7072614513158f210a7e69624a1aadd1603708f4f46564d8dd4195f87d
        bytes32 directory = keccak256("procedurePointer");
        // The case to uint240 drops the most significant bytes converting the value to
        // 0xd8dd4195f87d0000000000000000000000000000000000000000000000000000
        // then left shift the value 240 bits, losing all but the least signficant byte, the result is
        //   proc-prefix              procedure key
        // 0xd8dd4195f87d--000000000000000000000000000000000000000000000000
        // We than OR that with the key, which is 192 bits or 24 bytes. This is
        // the key provided on creation. If the key was 0x555555555555555555555555555555555555555555555555
        // then the resulting uint248 would be
        // 0xd8dd4195f87d5555555555555555555555555555555555555555555555555
        // TODO: it seems like this might not be what was intended
        return uint248(uint240(uint240(directory) << 192) | uint240(key)) << 8;

    }

    function _getProcedureByKey(uint192 key) internal view returns (Procedure memory p) {
        // pPointer is a uint248, which is all but one byte of a storage
        // address. This means that there are 256 storage keys "under"
        // this pPointer (at 32 bytes each this means 8,192 bytes of storage).
        uint248 pPointer = _getProcedurePointerByKey(key);
        // The first storage location (0) is used to store the keyIndex.
        p.keyIndex = uint8(_get(0, pPointer));
        // The second storage location (1) is used to store the address of the
        // contract.
        p.location = address(_get(0, pPointer + 1));
        // The thirs storage location (2) is used to store the number of caps
        uint256 nCaps = _get(0, pPointer + 2);
        p.caps = new Capability[](nCaps);
        // n is the cap index
        uint256 n = 0;
        // The rest of the 256 keys are (or can be) used for the caps
        for (uint248 i = 0; i < (256-3); i++) {
            if (n >= nCaps) {
                break;
            }
            uint256 thisCurrentCapLength = _get(0, pPointer+3+i);
            p.caps[n].capType = uint8(_get(0, pPointer+3+i+1));
            // subtract 1 from cap length because it includes the type
            uint256 nValues = thisCurrentCapLength - 1;
            // uint256 nValues = 2;
            p.caps[n].values = new uint256[](nValues);
            for (uint248 k = 0; k < nValues; k++) {
                p.caps[n].values[k] = uint256(_get(0, pPointer+3+i+2+k));
            }
            i = i + uint248(thisCurrentCapLength);
            n++;
        }
    }

    function checkRegisterCapability(Self storage /* self */, uint192 key, uint256 reqCapIndex) internal view returns (bool) {
        Procedure memory p = _getProcedureByKey(uint192(key));

        // If the requested cap is out of the bounds of the cap table, we
        // clearly don't have the capability;
        if ((p.caps.length == 0) || (reqCapIndex > (p.caps.length - 1))) {
            return false;
        }
        Capability memory cap = p.caps[reqCapIndex];
        // If the capability type is not REGISTER (11) it is the wrong type of
        // capability and we should reject
        if (cap.capType != CAP_PROC_REGISTER) {
            return false;
        }
        // If the cap is empty it implies all procedures are ok
        if (cap.values.length == 0) {
            return true;
        } else {
            // the register cap should always be empty, otherwise it is invalid
            return false;
        }
    }

    function checkDeleteCapability(Self storage /* self */, uint192 key, uint256 reqCapIndex) internal view returns (bool) {
        Procedure memory p = _getProcedureByKey(uint192(key));

        // If the requested cap is out of the bounds of the cap table, we
        // clearly don't have the capability;
        if ((p.caps.length == 0) || (reqCapIndex > (p.caps.length - 1))) {
            return false;
        }
        Capability memory cap = p.caps[reqCapIndex];
        // If the capability type is not DELETE it is the wrong type of
        // capability and we should reject
        if (cap.capType != CAP_PROC_DELETE) {
            return false;
        }
        // If the cap is empty it implies all procedures are ok
        if (cap.values.length == 0) {
            return true;
        } else {
            // the register cap should always be empty, otherwise it is invalid
            return false;
        }
    }

    function checkSetEntryCapability(Self storage /* self */, uint192 key, uint256 reqCapIndex) internal view returns (bool) {
        Procedure memory p = _getProcedureByKey(uint192(key));

        // If the requested cap is out of the bounds of the cap table, we
        // clearly don't have the capability;
        if ((p.caps.length == 0) || (reqCapIndex > (p.caps.length - 1))) {
            return false;
        }
        Capability memory cap = p.caps[reqCapIndex];
        // If the capability type is not ENTRY it is the wrong type of
        // capability and we should reject
        if (cap.capType != CAP_PROC_ENTRY) {
            return false;
        }
        // If the cap is empty it implies all procedures are ok
        if (cap.values.length == 0) {
            return true;
        } else {
            // the register cap should always be empty, otherwise it is invalid
            return false;
        }
    }

    function checkCallCapability(Self storage /* self */, uint192 key, bytes24 procedureKey, uint256 reqCapIndex) internal view returns (bool) {
        Procedure memory p = _getProcedureByKey(uint192(key));

        // If the requested cap is out of the bounds of the cap table, we
        // clearly don't have the capability;
        if ((p.caps.length == 0) || (reqCapIndex > (p.caps.length - 1))) {
            return false;
        }
        Capability memory cap = p.caps[reqCapIndex];
        // If the capability type is not CALL (0x3) it is the wrong type of
        // capability and we should reject
        if (cap.capType != CAP_PROC_CALL) {
            return false;
        }
        // If the cap is empty it implies all procedures are ok
        if (cap.values.length == 0) {
            return true;
        } else {
            // otherwise we cycle through the permitted procedure keys and see
            // if we can find the requested on
            for (uint256 i = 0; i < cap.values.length; i++) {
                if (bytes24(cap.values[i]/0x10000000000000000) == procedureKey) {
                    return true;
                }
            }
        }
    }

    function checkWriteCapability(Self storage /* self */, uint192 key, uint256 toStoreAddress, uint256 reqCapIndex) internal view returns (bool) {
        Procedure memory p = _getProcedureByKey(uint192(key));

        // If the requested cap is out of the bounds of the cap table, we
        // clearly don't have the capability;
        if ((p.caps.length == 0) || (reqCapIndex > (p.caps.length - 1))) {
            return false;
        }
        Capability memory cap = p.caps[reqCapIndex];
        uint256 capabilityType = cap.capType;
        // If the capability type is not WRITE (0x7) it is the wrong type of
        // capability and we should reject
        if (capabilityType != CAP_STORE_WRITE) {
            return false;
        }
        // We need two values for a valid write cap
        if (cap.values.length < 2) {
            return false;
        }
        uint256 capabilityKey = cap.values[0];
        uint256 capabilitySize = cap.values[1];

        if (capabilityType == CAP_STORE_WRITE
                && toStoreAddress >= capabilityKey
                && toStoreAddress <= (capabilityKey + capabilitySize)) {
            return true;
        }
    }

    function checkLogCapability(Self storage /* self */, uint192 key, bytes32[] reqTopics, uint256 reqCapIndex) internal view returns (bool) {
        Procedure memory p = _getProcedureByKey(uint192(key));

        // If the requested cap is out of the bounds of the cap table, we
        // clearly don't have the capability;
        if ((p.caps.length == 0) || (reqCapIndex > (p.caps.length - 1))) {
            return false;
        }
        Capability memory cap = p.caps[reqCapIndex];
        uint256 capabilityType = cap.capType;
        // If the capability type is not LOG (0x9) it is the wrong type of
        // capability and we should reject
        if (capabilityType != CAP_LOG) {
            return false;
        }
        // The number of topics is simply the number of keys of the cap (i.e.
        // not including the type)
        uint256 nTopics = cap.values.length;
        // Then we retrieve the topics
        bytes32[] memory capTopics = new bytes32[](nTopics);
        for (uint256 i = 0; i < nTopics; i++) {
            capTopics[i] = bytes32(cap.values[i]);
        }

        // Check that all of the topics required by the cap are satisfied. That
        // is, for every topic in the capability, the corresponding exists in
        // the system call and is set to that exact value. First we check that
        // there are enough topics in the request.
        if (reqTopics.length < capTopics.length) {
            // The system call specifies an insufficient number of topics
            return false;
        }
        for (uint256 j = 0; j < capTopics.length; j++) {
            if (reqTopics[j] != capTopics[j]) {
                return false;
            }
        }
        return true;
    }

    function _storeProcedure(Procedure memory p, uint192 key) internal {
        // Get the storage address of the name/key of the procedure. In this
        // scope "key" is the 24 byte name which is provided by the user. The
        // procedure "p" has already been given an index (p.keyIndex) which is
        // the offset where the name/key is stored.
        uint248 keyPointer = _getKeyPointerByIndex(p.keyIndex);
        // Store the name/key at this location.
        _set(0, keyPointer, key);

        // Get the storage address of the procedure data. This is the storage
        // key which contains all of the procedure data.
        uint248 pPointer = _getProcedurePointerByKey(key);
        _serialiseProcedure(p, 0, pPointer);
    }

    function _serialiseProcedure(Procedure memory p, uint8 storagePage, uint248 pPointer) internal {
        // Store the keyIndex at this location
        _set(storagePage, pPointer, p.keyIndex);
        // Store the address at the loction after this (making this data 2
        // uint256 wide).
        _set(storagePage, pPointer + 1, uint256(p.location));

        // The first value of the array is the number of capabilities
        _set(storagePage, pPointer + 2, p.caps.length);
        _serialiseCapArray(p, storagePage, pPointer);
    }

    function _serialiseCapArray(Procedure memory p, uint8 storagePage, uint248 pPointer) internal {
        // n is the storage key index
        uint248 n = 0;
        // i is the index of the cap
        for (uint248 i = 0; i < p.caps.length; i++) {
            uint256 nValues = p.caps[i].values.length;
            uint256 capSize = nValues + 1;
            _set(storagePage, pPointer + 3 + n, capSize); n++;
            _set(storagePage, pPointer + 3 + n, p.caps[i].capType); n++;
            for (uint248 j = 0; j < nValues; j++) {
                _set(storagePage, pPointer + 3 + n, p.caps[i].values[j]); n++;
            }
        }
    }

    function _freeProcedure(uint248 pPointer) internal {
        _set(0, pPointer, 0);
        _set(0, pPointer + 1, 0);
    }

    // Just returns an array of all the procedure data (257 32-byte values) concatenated.
    function returnRawProcedureTable(Self storage self) internal view returns (uint256[]) {
        bytes24[] memory keys = self.getKeys();
        uint256 len = keys.length;
        // max is 256 keys times the number of procedures
        uint256[] memory r = new uint256[](len*257);
        // The rest are the elements
        uint256 n = 0;
        for (uint248 i = 0; i < len ; i++) {
            uint192 key = uint192(keys[i]);
            uint248 pPointer = _getProcedurePointerByKey(key);
            r[n] = uint256(key); n++;
            for (uint248 j = 0; j < 256; j++) {
                r[n] = _get(0, pPointer+j); n++;
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
        uint256 n = 0;
        for (uint248 i = 0; i < len ; i++) {
            uint192 key = uint192(keys[i]);
            Procedure memory p = _getProcedureByKey(key);
            r[n] = uint256(key); n++;
            // Store the keyIndex at this location
            r[n] = p.keyIndex; n++;
            r[n] = uint256(p.location); n++;
            // number of capabilities in array
            r[n] = p.caps.length; n++;
            // cycle through each capability
            for (uint248 j = 0; j < p.caps.length; j++) {
                // we add 1 to account for the capType
                uint256 capSize = p.caps[j].values.length + 1;
                r[n] = capSize; n++;
                r[n] = p.caps[j].capType; n++;
                for (uint248 k = 0; k < p.caps[j].values.length; k++) {
                    r[n] = p.caps[j].values[k]; n++;
                }
            }
        }
        return r;
    }

    function insert(Self storage /* self */, bytes24 key, address value, uint256[] caps) internal returns (bool replaced) {
        // First we get retrieve the procedure that is specified by this key, if
        // it exists, otherwise the struct we create in memory is just
        // zero-filled.
        Procedure memory p = _getProcedureByKey(uint192(key));
        // We then write or overwrite the various properties
        p.location = value;
        // we just copy in the table in verbatim as long as its length is less
        // than 128 (arbitrary, but less than 256 minus room for other parameters)
        if (caps.length > 128) {
            revert();
        }

        // The capabilities are parsed here. We neeed to pass in the Procedure
        // struct as solidity can't return complex data structures.
        _parseCaps(p,caps);

        // If the keyIndex is not zero then that indicates that the procedure
        // already exists. In this case *WE HAVE NOT OVERWRITTEN * the values,
        // as we have not called _storeProcdure.
        if (p.keyIndex > 0) {
            return true;
        // If the keyIndex is zero (it is unsigned and cannot be negative) then
        // it means the procedure is new. We must therefore assign it a key
        // index.
        } else {
            // First we retrieve a pointer to the Procedure Table Length value.
            uint248 lenP = _getLengthPointer();
            // We then dereference that value.
            uint256 len = _get(0, lenP);
            // We assign this procedure the next keyIndex, i.e. len+1
            p.keyIndex = uint8(len + 1);
            // We increment the Procedure Table Length value
            _set(0, lenP, len + 1);
            // We actually commit the values in p to storage
            _storeProcedure(p, uint192(key));
            return false;
        }
    }

    // TODO: This should only add a single capability, currently it can add an
    // arbitrary number of caps.
    // TODO: Currently this just overwrites the caps.
    function addCap(Self storage /* self */, bytes24 key, uint256[] caps) internal returns (bool replaced) {
        // First we get retrieve the procedure that is specified by this key, if
        // it exists, otherwise the struct we create in memory is just
        // zero-filled.
        Procedure memory p = _getProcedureByKey(uint192(key));

        // we just copy in the table in verbatim as long as its length is less
        // than 128 (arbitrary, but less than 256 minus room for other parameters)
        if (caps.length > 128) {
            revert();
        }

        // The capabilities are parsed here. We neeed to pass in the Procedure
        // struct as solidity can't return complex data structures.
        _parseCaps(p,caps);

        // If the keyIndex is not zero then that indicates that the procedure
        // already exists. In this case *WE HAVE NOT OVERWRITTEN * the values,
        // as we have not called _storeProcdure.
        if (p.keyIndex > 0) {
            return true;
        // If the keyIndex is zero (it is unsigned and cannot be negative) then
        // it means the procedure is new. We must therefore assign it a key
        // index.
        } else {
            // First we retrieve a pointer to the Procedure Table Length value.
            uint248 lenP = _getLengthPointer();
            // We then dereference that value.
            uint256 len = _get(0, lenP);
            // We assign this procedure the next keyIndex, i.e. len+1
            p.keyIndex = uint8(len + 1);
            // We increment the Procedure Table Length value
            _set(0, lenP, len + 1);
            // We actually commit the values in p to storage
            _storeProcedure(p, uint192(key));
            return false;
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
        Procedure memory p1 = _getProcedureByKey(uint192(key));

        if (p1.keyIndex == 0)
            return false;
        // When we remove a procedure, we want to move another procedure into
        // this "slot" to keep the active keys contiguous
        // First we get the storage address of the Procedure Table Length
        uint248 lenP = _getLengthPointer();
        // We then dereference that to get the value
        uint256 len = _get(0, lenP);

        if (p1.keyIndex <= len) {
            // Move an existing element into the vacated key slot.
            uint248 p1P = _getProcedurePointerByKey(uint192(key));
            uint248 key1P = _getKeyPointerByIndex(p1.keyIndex);

            uint248 key2P = _getKeyPointerByIndex(uint8(len));
            uint192 key2 = uint192(_get(0, key2P));

            uint248 p2P = _getProcedurePointerByKey(key2);

            // This sets p2.keyIndex = p1.keyIndex
            _set(0, p2P, p1.keyIndex);
            _set(0, key1P, key2);

            // Free Old Key
            _set(0, key2P, 0);

            // Free P1
            _freeProcedure(p1P);

            // Decrement Keys Length
            _set(0, lenP, len - 1);

            return true;
        } else {
            return false;
        }
    }

    function contains(Self storage /* self */, bytes24 key) internal view returns (bool exists) {
        return _get(0, _getProcedurePointerByKey(uint192(key))) > 0;
    }

    function size(Self storage /* self */) internal view returns (uint) {
        return _get(0, _getLengthPointer());
    }

    function get(Self storage /* self */, bytes24 key) internal view returns (address) {
        return address(_get(0, _getProcedurePointerByKey(uint192(key)) + 1));
    }

    function getKeys(Self storage /* self */) internal view returns (bytes24[] memory keys) {
        uint248 lenP = _getLengthPointer();
        uint256 len = _get(0, lenP);
        keys = new bytes24[](len);
        for (uint248 i = 0; i < len; i += 1) {
            // We use +2 here because the name/key is the second uint256 store,
            // the first being the keyIndex.
            keys[i] = bytes24(_get(0, lenP + i + 2));
        }

    }

    function getKeyByIndex(Self storage /* self */, uint8 idx) internal view returns (uint192) {
        return uint192(_get(0, _getKeyPointerByIndex(idx)));
    }

    function getValueByIndex(Self storage self, uint8 idx) internal view returns (address) {
        return address(_get(0, _getProcedurePointerByKey(self.getKeyByIndex(idx)) + 1));
    }

}
