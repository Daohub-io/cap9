pragma solidity ^0.4.17;

library ProcedureTable {
    using ProcedureTable for ProcedureTable.Self;
    struct Self {
        // The table of procedures
        mapping(bytes32 => address) table;
    }

    function add(Self storage self, bytes32 name, address procedure) internal {
        self.table[name] = procedure;
    }

    function get(Self storage self, bytes32 name) internal returns (address p) {
        self.table[name];
    }
}