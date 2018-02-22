pragma solidity ^0.4.17;

library Processor {
    function create() private returns (address created) {
        assembly {
            // Get Size of Procedure
            let size := calldatasize()
            // Copy to Memory
            calldatacopy(0,0,size)
            // Deploy to Contract
            let created := create(1000, 0, size)
        }
    }
}