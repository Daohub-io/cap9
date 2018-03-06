pragma solidity ^0.4.17;

contract Factory {

    event LogFundsReceived(address sender, uint amount);
    event LogFundsSent(address receiver, uint amount);
    event LogContractCreation(bytes32 name, address location);

    mapping(bytes32 => address) public procedureTable;

    function() payable {
        LogFundsReceived(msg.sender, msg.value);
    }

    function codeLength(bytes oCode) public returns (uint len) {
        assembly {
            // Get Length
            len := mload(oCode)
        }
    }

    function codePosition(bytes oCode) public returns (uint code) {
        assembly {
            // Get Length
            let len := mload(oCode)
            // Get Code
            code := add(oCode, 0x00)
        }
    }

    function executeProcedure(bytes32 procedureName) public returns (uint v) {
        // Get the function address from the procedure table.
        var procedureAddress = procedureTable[procedureName];
        // Execute it using DELEGATECALL.
        procedureAddress.delegatecall(bytes4(sha3("add(uint,uint)")), 1,2);
        assembly {
            let returnSize = 32
            calldatacopy(0xff, 0, calldatasize)
            let _retVal = delegatecall(gas, currentVersion, 0xff, calldatasize, 0, returnSize)
            switch _retval case 0 { revert(0,0) } default { return(0, returnSize) }
        }
    }

    // As 'createAndPay', but will pay no gas into the contract.
    // TODO: this function should not be payable.
    function createProcedure(bytes32 procedureName, bytes oCode) payable public returns (address d) {
        return createProcedureAndPay(procedureName, 0, oCode);
    }

    // Will create a new contract of 'oCode' with gas of 'value'. It is up to
    // the the caller to ensure the factory has enough gas.
    function createProcedureAndPay(bytes32 procedureName, uint value, bytes oCode) payable public returns (address procedureAddress) {
        assembly {
            // Get length of code
            let len := mload(oCode)
            // Get position of code.
            let code := add(oCode, 0x20)

            // Deploy to Contract
            // Argument #1: The name (key) of the pocedure.
            // Argument #2: The amount of gas to be payed into the new contract
            //      on creation. Generally we do not want to do that, as we
            //      don't want contracts to hold gas.
            // Argument #3: The position of start of the code with an additional
            //      offset (as determined above).
            // Argument #4: The position of the end of the code
            //      (start + length).
            // Returns the address of the new contract. If gas is paid into the
            // new contract, but the factory doesn't hold enough gas, the null
            // address is returned.
            // TODO: catch null address returned here.
            procedureAddress := create(value, code, add(code, len))

            // Add address of the procedure to the table.
        }
        procedureTable[procedureName] = procedureAddress;
        LogContractCreation(procedureName, procedureAddress);
    }

    function validate(bytes oCode) public pure returns (bool valid) {
        for (uint8 i = 0; i < oCode.length; i++) {

            var ins = oCode[i];

            if (ins == 0xf1) {return false;}
            if (ins == 0xf2) {return false;}
            if (ins == 0xf3) {return false;}

            return true;
        }
    }

}