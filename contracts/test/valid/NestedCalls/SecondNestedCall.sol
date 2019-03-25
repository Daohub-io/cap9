pragma solidity ^0.4.17;

import "../../../BeakerContract.sol";

contract SecondNestedCall  is BeakerContract {


     // FirstNestedCall - store at 0x8001
     //   SecondNestedCall - store at 0x8002
     //     ThirdNestedCall - store at 0x8003
     //       FourthNestedCall - store at 0x8004
     //     FifthNestedCall - store at 0x8005
     //   SixthNestedCall - store at 0x8006
     // End
    function G() public {
        // First we do the store for FirstNestedCall
        write(1, 0x8002, 76);

        // Being our call to ThirdNestedCall
        bytes memory input1 = new bytes(4);
        bytes4 functionSelector1 = bytes4(keccak256("G()"));

        input1[0] = functionSelector1[0];
        input1[1] = functionSelector1[1];
        input1[2] = functionSelector1[2];
        input1[3] = functionSelector1[3];

        proc_call(2, "ThirdNestedCall", input1);

        // Being our call to FifthNestedCall
        bytes memory input2 = new bytes(4);
        bytes4 functionSelector2 = bytes4(keccak256("G()"));

        input2[0] = functionSelector2[0];
        input2[1] = functionSelector2[1];
        input2[2] = functionSelector2[2];
        input2[3] = functionSelector2[3];

        proc_call(2, "FifthNestedCall", input2);
    }
}
