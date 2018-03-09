pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    event KernelLog(string message);
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes32 name, bytes oCode) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }
        // Check whether the address exists
        address nullAddress;
        if (procedures.get(name) != nullAddress) {
            err = 3;
            return;
        }
        procedureAddress = create(oCode);
        procedures.add(name, procedureAddress);
    }

    function deleteProcedure(bytes32 name) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }
        // Check whether the address exists
        address nullAddress;
        procedureAddress = procedures.get(name);
        if (procedureAddress == nullAddress) {
            err = 3;
            return;
        }
        procedureAddress = procedures.remove(name);
    }

    function listProcedures() public view returns (bytes32[] listedKeys) {
        listedKeys = procedures.list();
    }

    function getProcedure(bytes32 name) public view returns (address procedureAddress) {
        procedureAddress = procedures.get(name);
    }

    function executeProcedure(bytes32 name, string fselector) public returns (uint8 err, uint256 retVal) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }
        // Check whether the address exists
        address nullAddress;
        address procedureAddress = procedures.get(name);
        if (procedureAddress == nullAddress) {
            err = 3;
            return;
        }
        bool status = false;
        assembly {
            // Store function selector string ("A()")
            // let m := mload(0x40)
            // mstore(0x40,add(m,32))
            // mstore(m, )

            // pop(mload(m))

            // Take the keccak256 has of that string, store at location n
            let n :=  mload(0x40)
            mstore(0x40,add(n,32))
            mstore(n,keccak256(add(fselector,0x20),mload(fselector))) // 3 is the length of the function selector string

            // Stores some
            let ins := n
            let inl := 0x4
            let outs := 0x60
            let outl := 0

            status := delegatecall(sub(gas,5000),procedureAddress,ins,inl,outs,outl)

        }
        if (!status) {
            err = 4;
        }
    }
}