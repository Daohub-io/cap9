pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }
        address nullAddress;
        if (procedures.get(name) != nullAddress) {
            err = 3;
            return;
        }
        procedureAddress = create(oCode);
        procedures.add(name, procedureAddress);
    }

    function deleteProcedure(bytes32 name) public returns (uint8 err, address procedureAddress) {

    }

    function listProcedures() public view returns (bytes32[] listedKeys) {
        listedKeys = procedures.list();
    }

    function getProcedure(bytes32 name) public view returns (address procedureAddress) {
        procedureAddress = procedures.get(name);
    }

    function executeProcedure(bytes32 name) public {}
}