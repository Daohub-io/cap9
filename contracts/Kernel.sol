pragma solidity ^0.4.17;


import "./Processor.sol";

contract Kernel {

    // Setup Processor
    // using Processor for Processor.Self;

    // address public newModule;
    address public owner;
    address public dog;
    
    function Kernel() public {
        owner = msg.sender;
    }

    function create(bytes memory oCode) public returns (address d) {
        // Get Size of Module
        var size = oCode.length;
        assembly {
            // Deploy to Contract
            let d := create(100, 0, size)
            sstore(owner_slot, d)
        }
    }






}