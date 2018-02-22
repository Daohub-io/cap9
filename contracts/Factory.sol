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
    }


}