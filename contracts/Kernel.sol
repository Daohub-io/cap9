pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
        }
        procedureAddress = create(name, oCode);
        procedures.add(name, procedureAddress);
    }

    function listProcedures() view returns (bytes32[] listedKeys) {
        listedKeys = procedures.list();
    }

    function getProcedure(bytes32 name) returns (address procedureAddress) {
        procedureAddress = procedures.get(name);
    }

    function executeProcedure(bytes32 name) {}
}