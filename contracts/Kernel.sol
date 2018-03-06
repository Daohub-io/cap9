pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) {
        var procedureAddress = create(name, oCode);
        procedures.add(name, procedureAddress);
    }

    function executeProcedure(bytes32 name) {}
}