pragma solidity ^0.4.17;

// This contract is a basic example of an entry procedure. It is used in
// testing. It simply executes any requested procedure.
contract BasicEntryProcedure {

    // This is fallback function for testing that simply logs something
    function () public {
        log0(bytes32("BasicEntryProcedureFallback"));
        // If there is no payload just exit
        if (msg.data.length == 0) {
            return;
        }
        // If there is a payload, assume the first 24 bytes are a procedure
        // key, and the rest is the payload. If there are not enough bytes,
        // throw an error
        if (msg.data.length < 24) {
            revert("no procedure key");
        }
        bytes24 procedureKey;
        assembly {
            procedureKey := calldataload(0)
        }
        // Call the requested procedure
        // Begin our call
        assembly {
            function malloc(size) -> result {
                // align to 32-byte words
                let rsize := add(size,sub(32,mod(size,32)))
                // get the current free mem location
                result :=  mload(0x40)
                // Bump the value of 0x40 so that it holds the next
                // available memory location.
                mstore(0x40,add(result,rsize))
            }
            let payloadLength := sub(calldatasize,24)
            let ins := malloc(add(0x80,payloadLength))
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(ins,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(ins,0x20),0x02)
            // The key of the procedure
            mstore(add(ins,0x40),procedureKey)

            // Copy the payload data into the input buffer
            calldatacopy(add(ins,0x80),24,payloadLength)

            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 97 because it is 1+32+32+32+4
            let status := delegatecall(gas, caller, add(ins,31), add(97,payloadLength), 0, 0)

            // Copy whatever was returned by the procedure into memory
            let retLoc := malloc(returndatasize)
            returndatacopy(retLoc,0,returndatasize)

            // Either return or revert that data (unchanged) depending on the
            // success of the procedure.
            if status {
                // success condition
                return(retLoc, returndatasize)
            }
            // error condition
            revert(retLoc, returndatasize)
        }
    }

    // This is simple function for testing that simply returns the number 37
    function A() public pure returns (uint256) {
        return 37;
    }

}
