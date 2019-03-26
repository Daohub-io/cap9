pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestCall is BeakerContract {
    // Log to no topics
    function A() public returns (uint32) {
        bytes memory input = new bytes(0);

        (uint32 err, /* bytes memory output */) = proc_call(2, "TestWrite", input);
        return err;
    }

    // Call SysCallTestWrite
    function B() public returns (uint32) {
        bytes memory input = new bytes(0);

        (uint32 err, /* bytes memory output */) = proc_call(2, "SysCallTestWrite", input);
        return err;
    }

    // Call SysTestCall:S()
    function C() public {
        bytes memory input = new bytes(4);
        bytes4 functionSelector = bytes4(keccak256("S()"));

        input[0] = functionSelector[0];
        input[1] = functionSelector[1];
        input[2] = functionSelector[2];
        input[3] = functionSelector[3];

        proc_call(2, "SysCallTestWrite", input);
    }

    // Call Adder:add(3,5), return result
    function E() public returns (uint) {

        bytes4 functionSelector = bytes4(keccak256("add(uint256,uint256)"));
        bytes memory input = new bytes(68);

        input[0] = functionSelector[0];
        input[1] = functionSelector[1];
        input[2] = functionSelector[2];
        input[3] = functionSelector[3];

        input[35] = 0x03;
        input[67] = 0x05;

        (/* uint32 err */, bytes memory output) = proc_call(2, "Adder", input);

        return uint256(output[31]);
    }

    // Do deeper call stacks
    function F() public returns (uint256) {

        // We will store the result from the first procedure call (add) here
        bytes4 functionSelector = bytes4(keccak256("add(uint256,uint256)"));
        bytes memory input = new bytes(68);

        input[0] = functionSelector[0];
        input[1] = functionSelector[1];
        input[2] = functionSelector[2];
        input[3] = functionSelector[3];

        input[35] = 0x03;
        input[67] = 0x05;

        (/* uint32 err */, bytes memory output) = proc_call(2, "Adder", input);
        uint256 addResult = uint256(output[31]);

        bytes4 functionSelector2 = bytes4(keccak256("S()"));
        bytes memory input2 = new bytes(4);

        input2[0] = functionSelector2[0];
        input2[1] = functionSelector2[1];
        input2[2] = functionSelector2[2];
        input2[3] = functionSelector2[3];

        proc_call(2, "SysCallTestWrite", input2);

        return addResult;
    }
}
