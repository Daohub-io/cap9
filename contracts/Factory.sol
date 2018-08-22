pragma solidity ^0.4.17;

contract Factory {

    /*opCode -> jump size*/
    mapping(byte => uint8) public opCodes;

    // event LogFundsReceived(address sender, uint amount);
    // event LogFundsSent(address receiver, uint amount);

    // function() payable public {
    //     LogFundsReceived(msg.sender, msg.value);
    // }

    function validate(bytes oCode) public view returns (uint8 err) {
        for (uint256 i = 0; i < oCode.length; i ++) {
            uint8 ins = uint8(oCode[i]);

            if (ins == 0xf0) {return 1;} // CREATE
            if (ins == 0xf1) {return 2;} // CALL
            if (ins == 0xf2) {return 3;} // CALLCODE
            if (ins == 0xf4) {return 4;} // DELEGATECALL
            if (ins == 0xff) {return 5;} // SUICIDE

            if (ins >= 0x60 && ins <= 0x7f) {
                i += ins - 95;
            }
        }
        return 0;
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
