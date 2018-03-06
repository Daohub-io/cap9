pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) returns (address procedureAddress) {
        procedureAddress = create(name, oCode);
        procedures.add(name, procedureAddress);
    }

    // function getProcedure(bytes32 name, bytes oCode) returns (address procedureAddress)

    function executeProcedure(bytes32 name) {}
}