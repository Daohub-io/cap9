pragma solidity ^0.4.17;

contract Factory {

    function create(bytes oCode) public returns (address d) {
        assembly {
            // Get Length
            let len := mload(oCode)
            // Get Code
            let code := add(oCode, 0x20)
            
            // Deploy to Contract
            d := create(20000000, code, add(code, len))
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