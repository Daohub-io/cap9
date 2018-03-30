pragma solidity ^0.4.17;

contract Factory {

    /*opCode -> jump size*/
    mapping(byte => uint8) public opCodes;

    event LogFundsReceived(address sender, uint amount);
    event LogFundsSent(address receiver, uint amount);

    function() payable public {
        LogFundsReceived(msg.sender, msg.value);
    }

    function Factory() public {
        /* PUSH opCodes */
        // TODO: replace with code
        opCodes[0x60] = 1;  opCodes[0x61] = 2;  opCodes[0x62] = 3;  opCodes[0x63] = 4;
        opCodes[0x64] = 5;  opCodes[0x65] = 6;  opCodes[0x66] = 7;  opCodes[0x67] = 8;
        opCodes[0x68] = 9;  opCodes[0x69] = 10; opCodes[0x6a] = 11; opCodes[0x6b] = 12;
        opCodes[0x6c] = 13; opCodes[0x6d] = 14; opCodes[0x6e] = 15; opCodes[0x6f] = 16;
        opCodes[0x70] = 17; opCodes[0x71] = 18; opCodes[0x72] = 19; opCodes[0x73] = 20;
        opCodes[0x74] = 21; opCodes[0x75] = 22; opCodes[0x76] = 23; opCodes[0x77] = 24;
        opCodes[0x78] = 25; opCodes[0x79] = 26; opCodes[0x7a] = 27; opCodes[0x7b] = 28;
        opCodes[0x7c] = 29; opCodes[0x7d] = 30; opCodes[0x7e] = 31; opCodes[0x7f] = 32;
    }

    function validate(bytes oCode) public view returns (bool valid) {
        uint256 lowerLimit = 0x0100000000000000000000000000000000000000000000000000000000000000;
        uint256 upperLimit = 0x0200000000000000000000000000000000000000000000000000000000000000;

        for (uint256 i = 0; i < oCode.length; i ++) {
            byte ins = oCode[i];

            if (ins == 0xf0 || // CREATE
                ins == 0xf1 || // CALL
                ins == 0xf2 || // CALLCODE
                ins == 0xf4 || // DELEGATECALL
                ins == 0xff)   // SUICIDE
            {
                return false;
            }
            if (ins == 0x55 /* SSTORE */) {
                // If an SSTORE opcode is found, check that the preceding
                // opcodes form a valid protection.
                if (
                    i < 73 ||
                oCode[i-73] != 0x7f ||
                // lower limit address is checked below
                oCode[i-40] != 0x81 ||
                oCode[i-39] != 0x10 ||
                oCode[i-38] != 0x7f ||
                // upper limit address is checked below
                oCode[i-5] != 0x82 ||
                oCode[i-4] != 0x11 ||
                oCode[i-3] != 0x17 ||
                oCode[i-2] != 0x58 ||
                oCode[i-1] != 0x57) return false;

                // Check the lower and upper address limits
                for (uint8 j = 0; j < 32; j ++) {
                    if (oCode[i-72+j] != bytes1(lowerLimit >> (248 - j*8)) || // lower limit
                        oCode[i-37+j] != bytes1(upperLimit >> (248 - j*8))    // upper limit
                        ) {
                        return false;
                    }
                }
            }


            i = i + opCodes[ins];
        }
        return true;
    }

    function codeLength(bytes oCode) pure public returns (uint len) {
        assembly {
            // Get Length
            len := mload(oCode)
        }
    }

    function codePosition(bytes oCode) pure public returns (uint code) {
        assembly {
            // Get Length
            let len := mload(oCode)
            // Get Code
            code := add(oCode, 0x00)
        }
    }

    // As 'createAndPay', but will pay no gas into the contract.
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
    function create(bytes oCode) public returns (address d) {
        assembly {
            // Get length of code
            let len := mload(oCode)
            // Get position of code.
            let code := add(oCode, 0x20)

            d := create(0, code, add(code, len))
        }
    }
}
