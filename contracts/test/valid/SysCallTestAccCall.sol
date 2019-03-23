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
        // bytes4 functionSelector = bytes4(keccak256("add(uint256,uint256)"));
        // 771602f7
        bytes memory input = new bytes(68);

        // input[0] = functionSelector[0];
        input[0] = 0x77;
        input[1] = 0x16;
        input[2] = 0x02;
        input[3] = 0xf7;

        input[35] = 0x03;
        input[67] = 0x05;

        (/* uint32 err */, bytes memory output) = proc_acc_call(2, account, amount, input);

        return uint256(output[31]);
    }

}
