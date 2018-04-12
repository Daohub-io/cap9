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
        return pointer & (uint256(fileId) << 24);
    }

    function _get(uint8 fileId, uint256 _pointer) internal returns (uint256 val) {
        var pointer = _filePointer(fileId, _pointer);
        assembly {
            // Load Value
            val := sload(pointer)
        }
    }

    function _set(uint8 fileId, uint256 _pointer, uint256 value) internal {
        // Convert Mask to Uint256
        var pointer = _filePointer(fileId, _pointer);
        assembly {
            sstore(pointer, value)
        }
    }

    function _getLengthPointer() internal returns (uint256) {
        return uint256(keccak256("keyPointer"));
    }
    function _getKeyPointerByIndex(uint8 idx) internal returns (uint256) {
        return uint256(keccak256("keyPointer")) + 1 + uint256(idx);
    }

    function _getProcedurePointerByKey(uint256 key) internal returns (uint256) {
        return uint256(keccak256("procedurePointer")) + key;
    }

    function _getProcedureByKey(uint256 key) internal returns (Procedure memory p) {
        uint256 pPointer = _getProcedurePointerByKey(key);
        p.keyIndex = uint8(_get(0, pPointer));
        p.location = address(_get(0, pPointer + 1));
    }

    function _storeProcedure(Procedure memory p, uint256 key) internal {
        uint256 keyPointer = _getKeyPointerByIndex(p.keyIndex);
        uint256 pPointer = _getProcedurePointerByKey(key);
        _set(0, keyPointer, key);
        _set(0, pPointer, p.keyIndex);
        _set(0, pPointer + 1, uint256(p.location));
    }

    function _freeProcedure(uint256 pPointer) internal {
        _set(0, pPointer, 0);
        _set(0, pPointer + 1, 0);
    }

    function insert(Self storage self, bytes32 key, address value) internal returns (bool replaced) {
        Procedure memory p = _getProcedureByKey(uint256(key));
        p.location = value;
        if (p.keyIndex > 0) {
            return true;
        } else {
            uint256 lenP = _getLengthPointer();
            uint256 len = _get(0, lenP);
            p.keyIndex = uint8(len + 1);
            _set(0, lenP, len + 1);
            _storeProcedure(p, uint256(key));
            return false;
        }
    }

    function remove(Self storage self, bytes32 key) internal returns (bool success) {
        Procedure memory p1 = _getProcedureByKey(uint256(key));
        
        if (p1.keyIndex == 0)
            return false;
        
        uint256 lenP = _getLengthPointer();
        uint256 len = _get(0, lenP);

        if (p1.keyIndex <= len) {
            // Move an existing element into the vacated key slot.
            uint256 p1P = _getProcedurePointerByKey(uint256(key));
            uint256 key1P = _getKeyPointerByIndex(p1.keyIndex);

            uint256 key2P = _getKeyPointerByIndex(uint8(len-1));
            uint256 key2 = _get(0, key2P);

            uint256 p2P = _getProcedurePointerByKey(key2);

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

    function contains(Self storage self, bytes32 key) internal constant returns (bool exists) {
        return _get(0, _getProcedurePointerByKey(uint256(key))) > 0;
    }

    function size(Self storage self) internal constant returns (uint) {
        return _get(0, _getLengthPointer());
    }

    function get(Self storage self, bytes32 key) internal constant returns (address) {
        return address(_get(0, _getProcedurePointerByKey(uint256(key)) + 1));
    }

    function getKeys(Self storage self) internal constant returns (bytes32[] memory keys) {
        uint256 lenP = _getLengthPointer();
        uint256 len = _get(0, lenP);
        for (uint256 i = 0; i < len; i += 1) {
            keys[i] = bytes32(_get(0, lenP + i + 1));
        }
    }

    function getKeyByIndex(Self storage self, uint8 idx) internal constant returns (uint256) {
        return _get(0, _getKeyPointerByIndex(idx));
    }

    function getValueByIndex(Self storage self, uint8 idx) internal constant returns (address) {
        return address(_get(0, _getProcedurePointerByKey(self.getKeyByIndex(idx)) + 1));
    }
    
}