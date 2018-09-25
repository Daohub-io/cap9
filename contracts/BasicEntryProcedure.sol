pragma solidity ^0.4.17;

// This contract is a basic example of an entry procedure. It is used in
// testing. It simply executes any requested procedure.
contract BasicEntryProcedure {

    // This is fallback function for testing that simply logs something
    function () public {
        log0(bytes32("BasicEntryProcedureFallback"));
        // If there is not payload just exit
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
        // log1(bytes32(procedureKey), bytes32("KeyName"));
        // log1(bytes32(msg.data.length-24), bytes32("Payload Length"));
        // for (uint256 i = 24; i < msg.data.length; i++) {
        //     log1(bytes32(msg.data[i]), bytes32("Payload"));
        // }
        bytes memory payload = msg.data;
        // Call the requested procedure
        // Begin our call
        bytes32 res;
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
            let ins := malloc(128)
            // First set up the input data (at memory location 0x0)
            // The call call is 0x-03
            mstore(add(ins,0x0),0x03)
            // The capability index is 0x-02
            mstore(add(ins,0x20),0x02)
            // The key of the procedure
            mstore(add(ins,0x40),procedureKey)
            // The size of the return value we expect (0x20)
            let retSize := 0x20
            let retLoc := malloc(retSize)
            mstore(add(ins,0x60),retSize)

            // Copy the payload data into the input buffer
            // let payloadLength := sub(calldataload(0),24)
            calldatacopy(add(ins,0x80),24,4)
            // log0(add(ins,0x80),4)
            // "in_offset" is at 31, because we only want the last byte of type
            // "in_size" is 65 because it is 1+32+32+32+4
            // we will store the result at 0x80 and it will be 32 bytes
            if iszero(delegatecall(gas, caller, add(ins,31), 101, retLoc, retSize)) {
                mstore(retLoc,add(2200,mload(retLoc)))
                return(retLoc,retSize)
            }
            res := mload(retLoc)
            return(retLoc,retSize)
        }
        // log0(res);
        // log0(bytes32("call complete"));
        // End procedure call
    }

    // This is simple function for testing that simply returns the number 37
    function A() public pure returns (uint256) {
        return 37;
    }

}
