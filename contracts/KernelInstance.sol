pragma solidity ^0.4.17;

import "./Kernel.sol";

contract KernelInstance is Kernel {
    constructor(address initProcAddress) public {
        // Set the kernel address to the current address.
        _setKernelAddress(this);

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
}
