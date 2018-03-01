pragma solidity ^0.4.17;

contract Factory {

    function codeLength(bytes oCode) public returns (uint len) {
        assembly {
            // Get Length
            len := mload(oCode)
        }
        return len;
    }

    function codePosition(bytes oCode) public returns (uint code) {
        assembly {
            // Get Length
            let len := mload(oCode)
            // Get Code
            code := add(oCode, 0x00)
        }
        return code;
    }

    function create(bytes oCode) public returns (address d) {
        assembly {
            // Get Length
            let len := mload(oCode)
            // Get Code
            let code := add(oCode, 0x20)

            // Deploy to Contract
            // TODO: For some reason the first parameter must be zero.
            d := create(1, code, add(code, len))
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