pragma solidity ^0.4.17;

contract Factory {

    /*opCode -> jump size*/
    mapping(byte => uint8) public opCodes;

    function validateContract(address procedureAddress) public returns (bool) {
        uint256 codeSize = 0;
        assembly {
            codeSize := extcodesize(procedureAddress)
        }
        bytes memory code = new bytes(codeSize);
        assembly {
            mstore(code, codeSize)
            extcodecopy(procedureAddress, add(code,0x20), 0, extcodesize(procedureAddress))
        }
        uint8 validationResult = validate(code);
        if (validationResult == 0) {
            return true;
        } else {
            return false;
        }
    }

    function validate(bytes oCode) public view returns (uint8 err) {
        for (uint256 i = 0; i < oCode.length; i ++) {
            uint8 ins = uint8(oCode[i]);
            // TODO: this also checks the swarm metadata, which is not actually
            // meant to be executed. We can't just skip it, as it is possible
            // to include code hidden in this metadata. We can either force
            // developers to remove it before being entered as a procedure, or
            // somehow try and prove that it is unreachable, which is possible
            // but an extra cost we probably don't want.
            //
            // It is always possible for legitimate swarm metadata to have a
            // jump destination which could be used to embed executable code.
            // It is not generally possible to determine what is a legitmate
            // swarm hash which just happens to contain a jump destination and
            // what is malicious code.

            // This presented as a whitelist in case any new state-changing
            // opcodes are added (CREATE2 being a good example)
            if (ins == 0x00) {continue;} // STOP
            if (ins == 0x01) {continue;} // ADD
            if (ins == 0x02) {continue;} // MUL
            if (ins == 0x03) {continue;} // SUB
            if (ins == 0x04) {continue;} // DIV
            if (ins == 0x05) {continue;} // SDIV
            if (ins == 0x06) {continue;} // MOD
            if (ins == 0x07) {continue;} // SMOD
            if (ins == 0x08) {continue;} // ADDMOD
            if (ins == 0x09) {continue;} // MULMOD
            if (ins == 0x0a) {continue;} // EXP
            if (ins == 0x0b) {continue;} // SIGNEXTEND

            if (ins == 0x10) {continue;} // LT
            if (ins == 0x11) {continue;} // GT
            if (ins == 0x12) {continue;} // SLT
            if (ins == 0x13) {continue;} // SGT
            if (ins == 0x14) {continue;} // EQ
            if (ins == 0x15) {continue;} // ISZERO
            if (ins == 0x16) {continue;} // AND
            if (ins == 0x17) {continue;} // OR
            if (ins == 0x18) {continue;} // XOR
            if (ins == 0x19) {continue;} // NOT
            if (ins == 0x1a) {continue;} // BYTE

            if (ins == 0x20) {continue;} // SHA3

            if (ins == 0x30) {continue;} // ADDRESS
            if (ins == 0x31) {continue;} // BALANCE
            if (ins == 0x32) {continue;} // ORIGIN
            if (ins == 0x33) {continue;} // CALLER
            if (ins == 0x34) {continue;} // CALLVALUE
            if (ins == 0x35) {continue;} // CALLDATALOAD
            if (ins == 0x36) {continue;} // CALLDATASIZE
            if (ins == 0x37) {continue;} // CALLDATACOPY
            if (ins == 0x38) {continue;} // CODESIZE
            if (ins == 0x39) {continue;} // CODECOPY
            if (ins == 0x3a) {continue;} // GASPRICE
            if (ins == 0x3b) {continue;} // EXTCODESIZE
            if (ins == 0x3c) {continue;} // EXTCODECOPY
            if (ins == 0x3d) {continue;} // RETURNDATASIZE
            if (ins == 0x3e) {continue;} // RETURNDATACOPY

            if (ins == 0x40) {continue;} // BLOCKHASH
            if (ins == 0x41) {continue;} // COINBASE
            if (ins == 0x42) {continue;} // TIMESTAMP
            if (ins == 0x43) {continue;} // NUMBER
            if (ins == 0x44) {continue;} // DIFFICULTY
            if (ins == 0x40) {continue;} // DIFFICULTY

            if (ins == 0x50) {continue;} // POP
            if (ins == 0x51) {continue;} // MLOAD
            if (ins == 0x52) {continue;} // MSTORE
            if (ins == 0x53) {continue;} // MSTORE8
            if (ins == 0x54) {return 1;} // SLOAD
            if (ins == 0x55) {return 2;} // SSTORE
            if (ins == 0x56) {continue;} // JUMP
            if (ins == 0x57) {continue;} // JUMPI
            if (ins == 0x58) {continue;} // PC
            if (ins == 0x59) {continue;} // MSIZE
            if (ins == 0x5a) {continue;} // GAS
            if (ins == 0x5b) {continue;} // JUMPDEST

            if (ins >= 0x60 && ins <= 0x7f) {
                i += ins - 95;
            }

            if (ins == 0x80) {continue;} // DUP1
            if (ins == 0x81) {continue;} // DUP2
            if (ins == 0x82) {continue;} // DUP3
            if (ins == 0x83) {continue;} // DUP4
            if (ins == 0x84) {continue;} // DUP5
            if (ins == 0x85) {continue;} // DUP6
            if (ins == 0x86) {continue;} // DUP7
            if (ins == 0x87) {continue;} // DUP8
            if (ins == 0x88) {continue;} // DUP9
            if (ins == 0x89) {continue;} // DUP10
            if (ins == 0x8a) {continue;} // DUP11
            if (ins == 0x8b) {continue;} // DUP12
            if (ins == 0x8c) {continue;} // DUP13
            if (ins == 0x8d) {continue;} // DUP14
            if (ins == 0x8e) {continue;} // DUP15
            if (ins == 0x8f) {continue;} // DUP16

            if (ins == 0x90) {continue;} // SWAP1
            if (ins == 0x91) {continue;} // SWAP2
            if (ins == 0x92) {continue;} // SWAP3
            if (ins == 0x93) {continue;} // SWAP4
            if (ins == 0x94) {continue;} // SWAP5
            if (ins == 0x95) {continue;} // SWAP6
            if (ins == 0x96) {continue;} // SWAP7
            if (ins == 0x97) {continue;} // SWAP8
            if (ins == 0x98) {continue;} // SWAP9
            if (ins == 0x99) {continue;} // SWAP10
            if (ins == 0x9a) {continue;} // SWAP11
            if (ins == 0x9b) {continue;} // SWAP12
            if (ins == 0x9c) {continue;} // SWAP13
            if (ins == 0x9d) {continue;} // SWAP14
            if (ins == 0x9e) {continue;} // SWAP15
            if (ins == 0x9f) {continue;} // SWAP16

            if (ins == 0xa0) {return 3;} // LOG0
            if (ins == 0xa1) {return 4;} // LOG1
            if (ins == 0xa2) {return 5;} // LOG2
            if (ins == 0xa3) {return 6;} // LOG3
            if (ins == 0xa4) {return 7;} // LOG4

            if (ins == 0xf0) {return 8;} // CREATE
            if (ins == 0xf1) {return 9;} // CALL
            if (ins == 0xf2) {return 10;} // CALLCODE
            if (ins == 0xf3) {continue;} // RETURN
            if (ins == 0xf4) {return 11;} // DELEGATECALL
            if (ins == 0xf5) {return 12;} // CREATE2
            if (ins == 0xfa) {continue;} // STATICCALL
            if (ins == 0xfd) {continue;} // REVERT
            if (ins == 0xfe) {continue;} // INVALID
            if (ins == 0xff) {return 13;} // SELFDESTRUCT

        }
        return 0;
    }

    function codeLength(bytes oCode) public pure returns (uint len) {
        assembly {
            // Get Length
            len := mload(oCode)
        }
    }

    function codePosition(bytes oCode) public pure returns (uint code) {
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
