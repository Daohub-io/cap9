pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) returns (uint err, address procedureAddress) {
        if (name.length == 0) {
            err = 1;
        }
        procedureAddress = create(name, oCode);
        procedures.add(name, procedureAddress);
    }

    function getProcedure(bytes32 name) returns (address procedureAddress) {
        procedureAddress = procedures.get(name);
    }

    function executeProcedure(bytes32 name) {}
}