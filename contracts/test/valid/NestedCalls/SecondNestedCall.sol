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
        uint32[] memory input1 = new uint32[](1);
        input1[0] = 0;

        proc_call(2, "ThirdNestedCall", "G()", input1);

        // Being our call to FifthNestedCall
        uint32[] memory input2 = new uint32[](1);
        input2[0] = 0;
        proc_call(2, "FifthNestedCall", "G()", input2);
    }
}