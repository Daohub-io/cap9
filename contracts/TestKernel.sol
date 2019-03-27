pragma solidity ^0.4.17;

import "./Kernel.sol";

// Kernel Interface for Testing
contract TestKernel is Kernel {
    constructor() public {
        // kernelAddress = WhatIsMyAddress.get();
        // This is an example kernel global variable for testing.
        assembly {
            sstore(0x8000,3)
        }
    }

    function testGetter() public view returns(uint256) {
        assembly {
            mstore(0,sload(0x8000))
            return(0,0x20)
        }
    }

    function anyTestGetter(uint256 addr) public view returns(uint256) {
        assembly {
            mstore(0,sload(addr))
            return(0,0x20)
        }
    }

    function testSetter(uint256 value) public {
        assembly {
            sstore(0x8000,value)
        }
    }

    function setEntryProcedure(bytes24 key) public {
        _setEntryProcedure(key);
    }

    function getEntryProcedure() view public returns (uint192) {
        return _getEntryProcedure();
    }

    // Create a validated procedure.
    function registerProcedure(bytes24 name, address procedureAddress, uint256[] caps) public returns (uint8 err, address retAddress) {
        return _registerProcedure(name, procedureAddress, caps);
    }

    // Create a procedure without  going through any validation.
    function registerAnyProcedure(bytes24 name, address procedureAddress, uint256[] caps) public returns (uint8 err, address retAddress) {
        return _registerAnyProcedure(name, procedureAddress, caps);
    }

    function deleteProcedure(bytes24 name) public returns (uint8 err, address procedureAddress) {
        return _deleteProcedure(name);
    }

    function executeProcedure(bytes24 name, string fselector, bytes payload) public returns (uint8) {
        bytes memory result =  _executeProcedure(name, fselector, payload);
        return uint8(result[0]);
    }
}
