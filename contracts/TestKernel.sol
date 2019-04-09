pragma solidity ^0.4.17;

import "./Kernel.sol";

// Kernel Interface for Testing
contract TestKernel is Kernel {
    constructor(address initProcAddress) public {
        // This is an example kernel global variable for testing.
        assembly {
            sstore(0x8000,3)
        }
        uint256[] memory caps = new uint256[](21);
        // CAP_PROC_CALL            = 3;
        caps[0] = 3; // capSize
        caps[1] = 3; // capType
        caps[2] = 0; // capIndex
        // CAP_PROC_REGISTER        = 4;
        caps[3] = 3; // capSize
        caps[4] = 4; // capType
        caps[5] = 0; // capIndex
        // CAP_PROC_DELETE          = 5;
        caps[6] = 3; // capSize
        caps[7] = 5; // capType
        caps[8] = 0; // capIndex
        // CAP_PROC_ENTRY           = 6;
        caps[9] = 3; // capSize
        caps[10] = 6; // capType
        caps[11] = 0; // capIndex
        // CAP_STORE_WRITE          = 7;
        caps[12] = 3; // capSize
        caps[13] = 7; // capType
        caps[14] = 0; // capIndex
        // CAP_LOG                  = 8;
        caps[15] = 3; // capSize
        caps[16] = 8; // capType
        caps[17] = 0; // capIndex
        // CAP_ACC_CALL             = 9;
        caps[18] = 3; // capSize
        caps[19] = 9; // capType
        caps[20] = 0; // capIndex
        bytes24 procName = bytes24("init");
        // TODO: Using insert directly skips validation, we shouldn't do that
        insert(procName, initProcAddress, caps);
        _setEntryProcedureRaw(uint192(procName));
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
