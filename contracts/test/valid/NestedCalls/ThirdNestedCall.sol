pragma solidity ^0.4.17;

import "../../../BeakerContract.sol";

contract ThirdNestedCall  is BeakerContract {

     // FirstNestedCall - store at 0x8001
     //   SecondNestedCall - store at 0x8002
     //     ThirdNestedCall - store at 0x8003
     //       FourthNestedCall - store at 0x8004
     //     FifthNestedCall - store at 0x8005
     //   SixthNestedCall - store at 0x8006
     // End
    function G() public {
        // First we do the store for FirstNestedCall
        write(1, 0x8003, 77);

        // Being our call to FourthNestedCall
        bytes memory input = new bytes(4);
        bytes4 functionSelector = bytes4(keccak256("G()"));

        input[0] = functionSelector[0];
        input[1] = functionSelector[1];
        input[2] = functionSelector[2];
        input[3] = functionSelector[3];

        proc_call(2, "FourthNestedCall", input);
    }
}
