pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestAccCall is BeakerContract {

    function testNum() public pure returns (uint256) {
        return 7888;
    }

    // Test without any input data
    function A(address account, uint256 amount) public returns (bytes memory) {
        uint256[] memory input = new uint256[](0);
        var (err, output) = proc_acc_call(2, account, amount, input);
        return output;
    }

    // Call SysCallTestWrite
    function B() public returns (uint32) {
        uint32[] memory input = new uint32[](1);
        input[0] = 0;

        var (err, output) = proc_call(2, "SysCallTestWrite", "", input);
        return err;
    }

    // Call SysTestCall:S()
    function C() public {
        uint32[] memory input = new uint32[](1);
        input[0] = 0;

        var (err, output) = proc_call(2, "SysCallTestWrite", "S()", input);
    }

    // Call Adder:add(3,5), return result
    function E() public returns (uint) {

        uint32[] memory input = new uint32[](2);
        input[0] = 3;
        input[1] = 5;

        var (err, output) = proc_call(2, "Adder", "add(uint256,uint256)", input);

        return uint256(output[31]);
    }

    // Do deeper call stacks
    function F() public returns (uint256) {

        // We will store the result from the first procedure call (add) here
        uint32[] memory input = new uint32[](2);
        input[0] = 3;
        input[1] = 5;

        var (err, output) = proc_call(2, "Adder", "add(uint256,uint256)", input);
        uint256 addResult = uint256(output[31]);

        uint32[] memory input2 = new uint32[](1);
        input[0] = 0;

        proc_call(2, "SysCallTestWrite", "S()", input2);

        return addResult;
    }
}
