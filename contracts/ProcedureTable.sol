pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;

    struct Procedure {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
        bool capability;
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
    function _filePointer(uint8 fileId, uint256 pointer) internal returns (uint256) {
        // Mask to Uint256
        // Overwrite the most significant byte of pointer with fileId
        return uint256(pointer) | (uint256(fileId) << 248);
    }

    function _get(uint8 fileId, uint256 _pointer) internal view returns (uint256 val) {
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
        // The case to uint248 drops the least significant byte (0x7d) converting the value to
        // 0x85a94e7072614513158f210a7e69624a1aadd1603708f4f46564d8dd4195f8
        // then left shift the value 240 bits, losing all but the least signficant byte, the result is
        // 0xf8000000000000000000000000000000000000000000000000000000000000
        // We than OR that with the key, which is 192 bits or 24 bytes. This is
        // the key provided on creation. If the key was 0x555555555555555555555555555555555555555555555555
        // then the resulting uint248 would be
        // 0xf8000000000000555555555555555555555555555555555555555555555555
        // TODO: it seems like this might not be what was intended
        return (uint248(directory) << 240) | key;
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
        p.capability = bool(_get(0, pPointer + 2) != 0);
    }

    function getProcedureCapabilityByKey(Self storage self, uint192 key) internal returns (bool cap) {
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
        p.capability = bool(_get(0, pPointer + 2) != 0);
        cap = p.capability;
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
        uint256 cap;
        if (p.capability) {
            cap = 1;
        } else {
            cap = 0;
        }
        _set(0, pPointer + 2, cap);
    }

    function _freeProcedure(uint248 pPointer) internal {
        _set(0, pPointer, 0);
        _set(0, pPointer + 1, 0);
    }

    function insert(Self storage self, bytes24 key, address value, bool writeCap) internal returns (bool replaced) {
        // TODO: explain what this does
        Procedure memory p = _getProcedureByKey(uint192(key));
        p.location = value;
        p.capability = writeCap;
        if (p.keyIndex > 0) {
            return true;
        } else {
            uint248 lenP = _getLengthPointer();
            uint256 len = _get(0, lenP);
            p.keyIndex = uint8(len + 1);
            _set(0, lenP, len + 1);
            _storeProcedure(p, uint192(key));
            return false;
        }
    }

    function remove(Self storage self, bytes24 key) internal returns (bool success) {
        Procedure memory p1 = _getProcedureByKey(uint192(key));

        if (p1.keyIndex == 0)
            return false;

        uint248 lenP = _getLengthPointer();
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