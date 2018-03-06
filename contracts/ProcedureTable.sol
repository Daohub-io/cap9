pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;
    struct Self {
        // The table of procedures
        mapping(bytes32 => address) table;
        bytes32[] keys;
    }

    function list(Self storage self) internal view returns (bytes32[] listedKeys) {
        listedKeys = self.keys;
    }

    function add(Self storage self, bytes32 name, address procedure) internal {
        self.table[name] = procedure;
        self.keys.push(name);
    }

    function remove(Self storage self, bytes32 name) internal returns (address p) {
        // the bytecode will need to call self destruct, this will mean ensuring
        // any code submitted has a standard destructor.

    }

    function get(Self storage self, bytes32 name) internal view returns (address p) {
        p = self.table[name];
    }
}