pragma solidity ^0.4.17;

import "./Factory.sol";
import "./ProcedureTable.sol";

contract Kernel is Factory {
    event KernelLog(string message);
    using ProcedureTable for ProcedureTable.Self;
    ProcedureTable.Self procedures;

    function createProcedure(bytes24 name, bytes oCode) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name == 0) {
            err = 1;
            return;
        }

        // Check whether the address exists
        bool exist = procedures.contains(name);
        if (exist) {
            err = 3;
            return;
        }

        procedureAddress = create(oCode);
        procedures.insert(name, procedureAddress);
    }

    function deleteProcedure(bytes24 name) public returns (uint8 err, address procedureAddress) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }

        procedureAddress = procedures.get(name);
        bool success = procedures.remove(name);

        // Check whether the address exists
        if (!success) {err = 2;}
    }

    function listProcedures() public view returns (bytes24[] memory) {
        return procedures.getKeys();
    }

    // function nProcedures() public view returns (uint256) {
    //     return procedures.size();
    // }


    function getProcedure(bytes24 name) public view returns (address) {
        return procedures.get(name);
    }

    function executeProcedure(bytes24 name, string fselector, bytes payload) public returns (uint8 err, uint256 retVal) {
        // Check whether the first byte is null and set err to 1 if so
        if (name[0] == 0) {
            err = 1;
            return;
        }
        // Check whether the address exists
        bool exist = procedures.contains(name);
        if (!exist) {
            err = 3;
            return;
        }

        address procedureAddress = procedures.get(name);
        bool status = false;
        assembly {
            // Retrieve the address of new available memory from address 0x40
            let n :=  mload(0x40)
            // Replace the value of 0x40 with the next new available memory,
            // after the 4 bytes we will use to store the keccak hash.
            mstore(0x40,add(n,32))
            // Take the keccak256 hash of that string, store at location n
            // mstore
            // Argument #1: The address (n) calculated above, to store the
            //    hash.
            // Argument #2: The hash, calculted as follows:
            //   keccack256
            //   Argument #1: The location of the fselector string (which
            //     is simply the name of the variable) with an added offset
            //     of 0x20, as the first 0x20 is reserved for the length of
            //     the string.
            //   Argument #2: The length of the string, which is loaded from
            //     the first 0x20 of the string.
            mstore(n,keccak256(add(fselector,0x20),mload(fselector)))

            // The input starts at where we stored the hash (n)
            let ins := n
            // Currently that is only the function selector hash, which is 4
            // bytes long.
            let inl := 0x4
            // TODO: Allocate a new area of memory into which to write the
            // return data. This will depend on the size of the return type.
            let outs := 0x60
            // There is no return value, therefore it's length is 0 bytes long
            let outl := 0

            status := delegatecall(sub(gas,5000),procedureAddress,ins,inl,outs,outl)

        }
        if (!status) {
            err = 4;
        }
    }
}