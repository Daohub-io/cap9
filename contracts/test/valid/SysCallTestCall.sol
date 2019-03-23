pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestCall is BeakerContract {
    // Log to no topics
    function A() public returns (uint32) {
        uint32[] memory input = new uint32[](1);
        input[0] = 0;

        (uint32 err, /* bytes memory output */) = proc_call(2, "TestWrite", "", input);
        return err;
    }

    // Call SysCallTestWrite
    function B() public returns (uint32) {
        uint32[] memory input = new uint32[](1);
        input[0] = 0;

        (uint32 err, /* bytes memory output */) = proc_call(2, "SysCallTestWrite", "", input);
        return err;
    }

    // Call SysTestCall:S()
    function C() public {
        uint32[] memory input = new uint32[](1);
        input[0] = 0;

        proc_call(2, "SysCallTestWrite", "S()", input);
    }

    // Call Adder:add(3,5), return result
    function E() public returns (uint) {

        uint32[] memory input = new uint32[](2);
        input[0] = 3;
        input[1] = 5;

        (/* uint32 err */, bytes memory output) = proc_call(2, "Adder", "add(uint256,uint256)", input);

        return uint256(output[31]);
    }

    // Do deeper call stacks
    function F() public returns (uint256) {

        // We will store the result from the first procedure call (add) here
        uint32[] memory input = new uint32[](2);
        input[0] = 3;
        input[1] = 5;

        (/* uint32 err */, bytes memory output) = proc_call(2, "Adder", "add(uint256,uint256)", input);
        uint256 addResult = uint256(output[31]);

        uint32[] memory input2 = new uint32[](1);
        input[0] = 0;

        proc_call(2, "SysCallTestWrite", "S()", input2);

        return addResult;
    }
}
