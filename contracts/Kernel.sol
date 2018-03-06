pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel {
    using ProcedureTable for ProcedureTable.Self;
    Procedure.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) {
        var procedureAddress = Factory.createProcedure(oCode);
        procedures.add(name, procedureAddress);
    }

    function callProcedure(bytes32 name) {
        var procedureAddress = procedures.get(name);
        procedures.call(procedureAddress);
    }
}