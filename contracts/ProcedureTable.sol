pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;

    struct Procedure {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
        // The start of the capability array. The first value is the length
        uint248 capabilityArray;
        // TODO: pack into this array.
        Capability[] caps;
    }

    struct Capability {
        uint8 capType;
        uint256[] values;
    }

    struct Self {}

    // Convert Pointer To File Pointer
    // Takes a single byte and a full 256 bit storage location
    function _filePointer(uint8 fileId, uint248 pointer) internal returns (uint256) {
        // Mask to Uint256
        // Overwrite the most significant byte of pointer with fileId
        return uint256(pointer) | (uint256(fileId) << 248);
    }

    function _get(uint8 fileId, uint248 _pointer) internal view returns (uint256 val) {
        var pointer = _filePointer(fileId, _pointer);
        assembly {
            // Load Value
            val := sload(pointer)
        }
    }

    function _set(uint8 fileId, uint248 _pointer, uint256 value) internal {
        // Convert Mask to Uint256
        var pointer = _filePointer(fileId, _pointer);
        assembly {
            sstore(pointer, value)
        }
    }

    function _getLengthPointer() internal view returns (uint248) {
        var directory = bytes8(keccak256("keyPointer"));
        return uint248(directory) << 240;
    }
    function _getKeyPointerByIndex(uint8 idx) internal view returns (uint248) {
        bytes8 directory = bytes8(keccak256("keyPointer"));
        return (uint248(directory) << 240) + 1 + uint248(idx);
    }

    function _getProcedurePointerByKey(uint192 key) internal view returns (uint248) {
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
        // For now this is:
        // type + key + value
        //  8   + 32  +  32   = 72 bytes
        // Actually, for now it is simply a boolean to determine if the
        // procedure is allowed to write or not
        // TODO: I don't like the way we remove information here. If we want to
        // be consistent we should deserialise the capabilities. This feels
        // expensive to me.
        p.capabilityArray = uint248(pPointer + 2);
    }

    function checkWriteCapability(Self storage self, uint192 key, uint256 toStoreAddress, uint256 reqCapIndex) internal view returns (bool allow) {
        allow = false;
        // pPointer is a uint248, which is all but one byte of a storage
        // address. This means that there are 256 storage keys "under"
        // this pPointer (at 32 bytes each this means 8,192 bytes of storage).
        Procedure memory p = _getProcedureByKey(uint192(key));
        // The capability we have chosen is at capIndex
        // First we must find the capability at capIndex which means iterating
        // through the cap array
        uint248 capArrayPointer = p.capabilityArray;
        uint256 capIndex = 0;
        uint256 capabilityType = 0;
        uint256 capabilityKey = 0;
        uint256 capabilitySize = 0;


        for (uint248 i = 1; i < 256; i++) {
            uint256 capSize = _get(0, capArrayPointer+i+0);
            // if this is the relevant cap, then process it
            if (capIndex == reqCapIndex) {
                // process the cap
                capabilityType = _get(0, capArrayPointer+i+1);
                capabilityKey  = _get(0, capArrayPointer+i+2);
                capabilitySize = _get(0, capArrayPointer+i+3);
                break;
            } else {
                // skip the length of this cap
                i = i + uint248(capSize);
                capIndex++;
            }
        }
        if (capabilityType == 0x7 && toStoreAddress >= capabilityKey && toStoreAddress <= (capabilityKey + capabilitySize)) {
            allow = true;
        }
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
        // Store the keyIndex at this location
        _set(0, pPointer, p.keyIndex);
        // Store the address at the loction after this (making this data 2
        // uint256 wide).
        _set(0, pPointer + 1, uint256(p.location));
    }

    function _freeProcedure(uint248 pPointer) internal {
        _set(0, pPointer, 0);
        _set(0, pPointer + 1, 0);
    }

    function returnProcedureTable(Self storage self) internal view returns (uint256[]) {
        bytes24[] memory keys = self.getKeys();
        uint256 len = keys.length;
        // max is 256 keys times the number of procedures
        uint256[] memory r = new uint256[](len*256);
        // The rest are the elements
        for (uint248 i = 0; i < len ; i++) {
            uint248 n = i*256;
            uint192 key = uint192(keys[i]);
            uint248 pPointer = _getProcedurePointerByKey(key);
            r[n+0] = uint256(key);
            // Store the keyIndex at this location
            uint256 keyIndex = _get(0, pPointer+0);
            r[n+1] = keyIndex;
            uint256 location = _get(0, pPointer+1);
            r[n+2] = location;
            // number of capabilities in array
            uint256 capArrayLength = _get(0, pPointer+2);
            r[n+3] = capArrayLength;
            // cycle through each capability
            for (uint248 j = 0; j < capArrayLength; j++) {
                r[n+4+j] = _get(0, pPointer+3+j);
            }
        }
        return r;
    }

    function insert(Self storage self, bytes24 key, address value, uint256[] caps) internal returns (bool replaced) {
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
        // The first value of the array is the length of the capability array
        // (in keys)
        _set(0, p.capabilityArray, caps.length);
        // Set all the other bytes verbatim, with an offset of 1, as the first
        // value is used to store the length. We step through the capabilities
        // to ensure that the length values are correct.
        for (uint248 i = 0; i < caps.length; i++) {
            // Save the start offset of this cap, we add one, as the very first
            // value is the total length of the cap array
            uint248 startOffset = p.capabilityArray+1+i;
            // The first value is the length of the cap we are currently
            // processing.
            uint248 capLength = uint248(caps[i]);
            _set(0, startOffset+0, capLength);
            // Next we process this many keys.
            for (uint248 j = 1; j <= capLength; j++) {
                _set(0, startOffset+j, caps[i+j]);
            }
            i = i + capLength;
        }

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

    function remove(Self storage self, bytes24 key) internal returns (bool success) {
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

            uint248 key2P = _getKeyPointerByIndex(uint8(len-1));
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
        }
    }

    function contains(Self storage self, bytes24 key) internal view returns (bool exists) {
        return _get(0, _getProcedurePointerByKey(uint192(key))) > 0;
    }

    function size(Self storage self) internal view returns (uint) {
        return _get(0, _getLengthPointer());
    }

    function get(Self storage self, bytes24 key) internal view returns (address) {
        return address(_get(0, _getProcedurePointerByKey(uint192(key)) + 1));
    }

    function getKeys(Self storage self) internal view returns (bytes24[] memory keys) {
        uint248 lenP = _getLengthPointer();
        uint256 len = _get(0, lenP);
        keys = new bytes24[](len);
        for (uint248 i = 0; i < len; i += 1) {
            // We use +2 here because the name/key is the second uint256 store,
            // the first being the keyIndex.
            keys[i] = bytes24(_get(0, lenP + i + 2));
        }

    }

    function getKeyByIndex(Self storage self, uint8 idx) internal view returns (uint192) {
        return uint192(_get(0, _getKeyPointerByIndex(idx)));
    }

    function getValueByIndex(Self storage self, uint8 idx) internal view returns (address) {
        return address(_get(0, _getProcedurePointerByKey(self.getKeyByIndex(idx)) + 1));
    }

}
