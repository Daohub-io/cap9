pragma solidity ^0.4.17;

contract Factory {

    function validateContract(address procedureAddress) public view returns (uint8) {
        uint256 codeSize = 0;
        assembly {
            codeSize := extcodesize(procedureAddress)
        }
        bytes memory code = new bytes(codeSize);
        assembly {
            mstore(code, codeSize)
            extcodecopy(procedureAddress, add(code,0x20), 0, extcodesize(procedureAddress))
        }
        return validate(code);
    }

    function validate(bytes oCode) public pure returns (uint8 err) {
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

            if((ins >= 0x00 && ins <= 0x0b)){continue;}  // Stop and Arithmetic
            if((ins >= 0x10 && ins <= 0x1a)){continue;} // Comparison & Bitwise Logic Operations
            if((ins == 0x20)){continue;} // SHA3
            if((ins >= 0x30 && ins <= 0x3e)){continue;} // Environmental Informatio
            if((ins >= 0x40 && ins <= 0x45)){continue;} // Block Information
            if((ins >= 0x50 && ins <= 0x53)){continue;} // Stack, Memory, Storage and Flow Operation 
            if((ins >= 0x56 && ins <= 0x5b)){continue;} // Stack, Memory, Storage and Flow Operation
            if((ins >= 0x80 && ins <= 0x8f)){continue;} // Duplication Operations
            if((ins >= 0x90 && ins <= 0x9f)){continue;} // Exchange Operations
            if((ins == 0xf3)){continue;} // RETURN
            if((ins >= 0xfa && ins <= 0xfe)){continue;}

            if (ins >= 0x60 && ins <= 0x7f) {
                i += ins - 95;
                continue;
            } // PUSH

            // SLOAD is allowed
            if (ins == 0x54) {continue;} // SLOAD
            if (ins == 0x55) {return 2;} // SSTORE

            if (ins == 0xa0) {return 3;} // LOG0
            if (ins == 0xa1) {return 4;} // LOG1
            if (ins == 0xa2) {return 5;} // LOG2
            if (ins == 0xa3) {return 6;} // LOG3
            if (ins == 0xa4) {return 7;} // LOG4

            if (ins == 0xf0) {return 8;} // CREATE
            if (ins == 0xf1) {return 9;} // CALL
            if (ins == 0xf2) {return 10;} // CALLCODE
            if (ins == 0xf4) {
                // continue if it is a compliant syscall
                bool isSysCall = false;
                // check there are enough bytes
                if (i < 2) {
                    isSysCall = false;
                } else {
                    isSysCall = (oCode[i-1] == 0x5a /* GAS */) && (oCode[i-2] == 0x33 /* CALLER */);
                }
                if (isSysCall) {
                    continue;
                } else {
                    return 11;
                }
            } // DELEGATECALL
            if (ins == 0xf5) {return 12;} // CREATE2
            if (ins == 0xff) {return 13;} // SELFDESTRUCT

            

            return 100; // UNKNOWN OPCODE
            
        }
        return 0;
    }
}
