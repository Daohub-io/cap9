pragma solidity ^0.4.17;

import "../../BeakerContract.sol";

contract SysCallTestAccCall is BeakerContract {

    function testNum() public pure returns (uint256) {
        return 7888;
    }

    // Test without any input data
    function A(address account, uint256 amount) public returns (bytes memory) {
        bytes memory input = new bytes(0);
        (/* uint32 err */, bytes memory output) = proc_acc_call(2, account, amount, input);
        return output;
    }

    // Call Adder:add(3,5), return result
    function B(address account, uint256 amount) public returns (uint256) {
        bytes4 functionSelector = bytes4(keccak256("add(uint256,uint256)"));
        bytes memory input = new bytes(68);

        input[0] = functionSelector[0];
        input[1] = functionSelector[1];
        input[2] = functionSelector[2];
        input[3] = functionSelector[3];

        input[35] = 0x03;
        input[67] = 0x05;

        (/* uint32 err */, bytes memory output) = proc_acc_call(2, account, amount, input);

        return uint256(output[31]);
    }

}
