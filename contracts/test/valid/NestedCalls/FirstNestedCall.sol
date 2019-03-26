pragma solidity ^0.4.17;

import "../../../BeakerContract.sol";

contract FirstNestedCall is BeakerContract {
     // FirstNestedCall - store at 0x8001
     //   SecondNestedCall - store at 0x8002
     //     ThirdNestedCall - store at 0x8003
     //       FourthNestedCall - store at 0x8004
     //     FifthNestedCall - store at 0x8005
     //   SixthNestedCall - store at 0x8006
     // End
    function G() public {
        // First we do the store for FirstNestedCall
        write(1, 0x8001, 75);

        // Begin our call to SecondNestedCall
        bytes memory input = new bytes(4);
        bytes4 functionSelector = bytes4(keccak256("G()"));

        input[0] = functionSelector[0];
        input[1] = functionSelector[1];
        input[2] = functionSelector[2];
        input[3] = functionSelector[3];

        proc_call(2, "SecondNestedCall", input);

        // Being our call to SixthNestedCall
        bytes memory input2 = new bytes(4);
        bytes4 functionSelector2 = bytes4(keccak256("G()"));

        input2[0] = functionSelector2[0];
        input2[1] = functionSelector2[1];
        input2[2] = functionSelector2[2];
        input2[3] = functionSelector2[3];

        proc_call(2, "SixthNestedCall", input2);
    }
}
