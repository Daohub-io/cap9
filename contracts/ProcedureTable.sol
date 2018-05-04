pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;

    struct Procedure {
        // Equal to the index of the key of this item in keys, plus 1.
        uint8 keyIndex;
        address location;
    }

    struct Self {}

    // Convert Pointer To File Pointer

    function _filePointer(uint8 fileId, uint256 pointer) internal returns (uint256) {
        // Mask to Uint256
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
        var directory = keccak256("procedurePointer");
        return (uint248(directory) << 240) | key;
    }

    function _getProcedureByKey(uint192 key) internal returns (Procedure memory p) {
        uint248 pPointer = _getProcedurePointerByKey(key);
        p.keyIndex = uint8(_get(0, pPointer));
        p.location = address(_get(0, pPointer + 1));
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

    function insert(Self storage self, bytes24 key, address value) internal returns (bool replaced) {
        Procedure memory p = _getProcedureByKey(uint192(key));
        p.location = value;
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