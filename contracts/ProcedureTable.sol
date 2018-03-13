pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;
    struct Procedure {
        // Equal to the index of the key of this item in keys, plus 1.
        uint keyIndex;
        address location;
        
        // Storage Mask
        uint256 storageMask;
    }

    struct Self {
        mapping(bytes32 => Procedure) data;
        bytes32[] keys;
    }

    function insert(Self storage self, bytes32 key, address value, uint256 mask) internal returns (bool replaced) {
        Procedure storage p = self.data[key];
        p.location = value;
        p.storageMask = mask;

        if (p.keyIndex > 0) {
            return true;
        } else {
            p.keyIndex = ++self.keys.length;
            self.keys[p.keyIndex - 1] = key;
            return false;
        }
    }

    function remove(Self storage self, bytes32 key) internal returns (bool success) {
        Procedure storage p = self.data[key];
        if (p.keyIndex == 0)
            return false;

        if (p.keyIndex <= self.keys.length) {
            // Move an existing element into the vacated key slot.
            self.data[self.keys[self.keys.length - 1]].keyIndex = p.keyIndex;
            self.keys[p.keyIndex - 1] = self.keys[self.keys.length - 1];
            self.keys.length -= 1;
            delete self.data[key];
            return true;
        }
    }

    function destroy(Self storage self) internal {
        for (uint i; i<self.keys.length; i++) {
          delete self.data[self.keys[i]];
        }
        delete self.keys;
        return ;
    }

    function contains(Self storage self, bytes32 key) internal constant returns (bool exists) {
        return self.data[key].keyIndex > 0;
    }

    function size(Self storage self) internal constant returns (uint) {
        return self.keys.length;
    }

    function get(Self storage self, bytes32 key) internal constant returns (address) {
        return self.data[key].location;
    }

    function getKeyByIndex(Self storage self, uint idx) internal constant returns (bytes32) {
        return self.keys[idx];
    }

    function getValueByIndex(Self storage self, uint idx) internal constant returns (address) {
        return self.data[self.keys[idx]].location;
    }

}