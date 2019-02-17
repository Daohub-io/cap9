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
        uint32[] memory input = new uint32[](1);
        input[0] = 0;

        proc_call(2, "SecondNestedCall", "G()", input);

        // Being our call to SixthNestedCall
        uint32[] memory input2 = new uint32[](1);
        input[0] = 0;

        proc_call(2, "SixthNestedCall", "G()", input2);
    }
}