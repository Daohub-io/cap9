pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;

    struct Procedure {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
        // The type of capability (currently only WRITE 0x7)
        uint8 capabilityType;
        // The key to which we can write
        uint256 capabilityKey;
        // The number of additional keys we can write to
        uint256 capabilitySize;
    }

    // struct Capability {
    //     // Equal to the index of the key of this item in keys, plus 1.
    //     uint8 capType;
    //     uint256 storageLocation;
    //     uint256 storageValue;
    // }

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

    function _getLengthPointer() internal returns (uint248) {
        var directory = bytes8(keccak256("keyPointer"));
        return uint248(directory) << 240;
    }
    function _getKeyPointerByIndex(uint8 idx) internal returns (uint248) {
        var directory = bytes8(keccak256("keyPointer"));
        return (uint248(directory) << 240) + 1 + uint248(idx);
    }

    function _getProcedurePointerByKey(uint192 key) internal returns (uint248) {
        // Procedure data is stored under the procedurePointer "directory". The
        // location of the procedure data is followed by the name/key of the
        // procedure.
        // keccak256("procedurePointer") is 0x85a94e7072614513158f210a7e69624a1aadd1603708f4f46564d8dd4195f87d
        var directory = keccak256("procedurePointer");
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

    function _getProcedureByKey(uint192 key) internal returns (Procedure memory p) {
        // pPointer is a uint248, which is all but one byte of a storage
        // address. This means that there are 256 storage keys "under"
        // this pPointer (at 32 bytes each this means 8,192 bytes of storage).
        uint248 pPointer = _getProcedurePointerByKey(key);
        // The first storage location (0) is used to store the keyIndex.
        p.keyIndex = uint8(_get(0, pPointer));
        // The second storage location (1) is used to store the address of the
        // contract.
        p.location = address(_get(0, pPointer + 1));
        // TODO: add the capability list here. For now we only have a single
        // capability slot for a single capability type, which is write.
        // For now this is:
        // type + key + value
        //  8   + 32  +  32   = 72 bytes
        // Actually, for now it is simply a boolean to determine if the procedure
        // is allowed to write or not

        // The type of capability (currently only WRITE 0x7)
        p.capabilityType = uint8(_get(0, pPointer + 2));
        // The key to which we can write
        p.capabilityKey = uint256(_get(0, pPointer + 3));
        // The number of additional keys we can write to
        p.capabilitySize = uint256(_get(0, pPointer + 4));
    }

    function checkWriteCapability(Self storage self, uint192 key, uint256 toStoreAddress) internal returns (bool allow) {
        allow = false;
        // pPointer is a uint248, which is all but one byte of a storage
        // address. This means that there are 256 storage keys "under"
        // this pPointer (at 32 bytes each this means 8,192 bytes of storage).
        uint248 pPointer = _getProcedurePointerByKey(key);
        Procedure p;
        // The first storage location (0) is used to store the keyIndex.
        p.keyIndex = uint8(_get(0, pPointer));
        // The second storage location (1) is used to store the address of the
        // contract.
        p.location = address(_get(0, pPointer + 1));
        // TODO: add the capability list here. For now we only have a single
        // capability slot for a single capability type, which is write.
        // For now this is:
        // type + key + value
        //  8   + 32  +  32   = 72 bytes
        // Actually, for now it is simply a boolean to determine if the procedure
        // is allowed to write or not
        p.capabilityType = uint8(_get(0, pPointer + 2));
        p.capabilityKey = _get(0, pPointer + 3);
        p.capabilitySize = _get(0, pPointer + 4);
        if (p.capabilityType == 0x7 && toStoreAddress >= p.capabilityKey && toStoreAddress <= (p.capabilityKey + p.capabilitySize)) {
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
        // Store the write capability at the third location
        _set(0, pPointer + 2, p.capabilityType);
        _set(0, pPointer + 3, p.capabilityKey);
        _set(0, pPointer + 4, p.capabilitySize);
    }

    function _freeProcedure(uint248 pPointer) internal {
        _set(0, pPointer, 0);
        _set(0, pPointer + 1, 0);
    }

    function insert(Self storage self, bytes24 key, address value, uint8 capType, uint256 capAddress, uint256 capSize) internal returns (bool replaced) {
        // First we get retrieve the procedure that is specified by this key, if
        // it exists, otherwise the struct we create in memory is just
        // zero-filled.
        Procedure memory p = _getProcedureByKey(uint192(key));
        // We then write or overwrite the various properties
        p.location = value;
        p.capabilityType = capType;
        p.capabilityKey = capAddress;
        p.capabilitySize = capSize;
        // TODO: what does the code below do

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

    function contains(Self storage self, bytes24 key) internal constant returns (bool exists) {
        return _get(0, _getProcedurePointerByKey(uint192(key))) > 0;
    }

    function size(Self storage self) internal constant returns (uint) {
        return _get(0, _getLengthPointer());
    }

    function get(Self storage self, bytes24 key) internal constant returns (address) {
        return address(_get(0, _getProcedurePointerByKey(uint192(key)) + 1));
    }

    function getKeys(Self storage self) internal returns (bytes24[] memory keys) {
        uint248 lenP = _getLengthPointer();
        uint256 len = _get(0, lenP);
        keys = new bytes24[](len);
        for (uint248 i = 0; i < len; i += 1) {
            // We use +2 here because the name/key is the second uint256 store,
            // the first being the keyIndex.
            keys[i] = bytes24(_get(0, lenP + i + 2));
        }

    }

    function getKeyByIndex(Self storage self, uint8 idx) internal constant returns (uint192) {
        return uint192(_get(0, _getKeyPointerByIndex(idx)));
    }

    function getValueByIndex(Self storage self, uint8 idx) internal constant returns (address) {
        return address(_get(0, _getProcedurePointerByKey(self.getKeyByIndex(idx)) + 1));
    }

}
