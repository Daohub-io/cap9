pragma solidity ^0.4.17;

contract Factory {

    event LogFundsReceived(address sender, uint amount);
    event LogFundsSent(address receiver, uint amount);

    function() payable public {
        LogFundsReceived(msg.sender, msg.value);
    }

    function codeLength(bytes oCode) pure public returns (uint len) {
        assembly {
            // Get Length
            len := mload(oCode)
        }
        return len;
    }

    function codePosition(bytes oCode) pure public returns (uint code) {
        assembly {
            // Get Length
            let len := mload(oCode)
            // Get Code
            code := add(oCode, 0x00)
        }
        return code;
    }

    // As 'createAndPay', but will pay no gas into the contract.
    // TODO: this function should not be payable.
    function createProcedure(bytes oCode) payable public returns (address d) {
        return createProcedureAndPay(0, oCode);
    }

    // Will create a new contract of 'oCode' with gas of 'value'. It is up to
    // the the caller to ensure the factory has enough gas.
    function createProcedureAndPay(uint value, bytes oCode) payable public returns (address d) {
        assembly {
            // Get length of code
            let len := mload(oCode)
            // Get position of code.
            let code := add(oCode, 0x20)

            // Deploy to Contract
            // Argument #1: The amount of gas to be payed into the new contract
            //      on creation. Generally we do not want to do that, as we
            //      don't want contracts to hold gas.
            // Argument #2: The position of start of the code with an additional
            //      offset (as determined above).
            // Argument #3: The position of the end of the code
            //      (start + length).
            // Returns the address of the new contract. If gas is paid into the
            // new contract, but the factory doesn't hold enough gas, the null
            // address is returned.
            // TODO: catch null address returned here.
            d := create(value, code, add(code, len))
        }
        return d;
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