pragma solidity ^0.4.17;

contract Kernel {
    function() public {
        assembly {
            // Here we decode the system call (if there is one)
            switch div(calldataload(0), 0x10000000000000000000000000000000000000000000000000000000000)
            case 0 {
                // non-syscall case
                // here we need to use call. delegatecall would leave the CALLER as
                // the account which started the transaction
                // The address here needs to be updated to call Procedure1
                // everytime Procedure1 is deployed
                call(gas, 0x8885584aa73fccf0f4572a770d1a0d6bd0b4360a, 0, 29, 67, 0x80, 0x40)
                0x60
                mstore
                // return both delegatecall return values and the system call
                // retun value
                return(0x60,0x60)
            }
            // This is a store system call
            case 2 {
                let location := calldataload(3)
                let value := calldataload(add(3,32))
                sstore(location, value)
                mstore8(0xb0,3)
                log0(0xb0, 1)
                // sstore(0, div(x, 2))
            }
            default {
                mstore8(0xb0,5)
                log0(0xb0, 1)
            }

        }
    }
}

contract Procedure1 {
    function () public {
        assembly {
            // First set up the input data (at memory location 0x0)
            // The write call is 0x-00-00-02
            mstore(0x0,0x000002)
            // The storage location we want is 0x07
            mstore(0x20,0x07)
            // The value we want to store is 0x1234
            mstore(0x40,0x1234)
            // "in_offset" is at 29, because we only want the last 3 bytes
            // "in_size" is 67 because it is 3+32+32
            // we will store the result at 0x80 and it will be 1 byte
            delegatecall(gas, caller, 29, 67, 0x80, 1)
            mstore(0xd,sload(0x7))
            log0(0xd,32)
            // Store the result of delegatecall at 0x60
            0x60
            mstore
            // return both the delegatecall return value and the system call
            // retun value
            return(0x60,0x40)
        }
    }
}

contract Procedure2 {
    function docall() public returns (uint256) {
        assembly {
            call(gas, 0x35ef07393b57464e93deb59175ff72e6499450cf, 0, 0, 0, 0x10, 2)
            0

            mstore
            return(0,32)
        }
    }
}

contract Procedure3 {
    function docall() public returns (uint256) {
        assembly {
            call(gas, 0x35ef07393b57464e93deb59175ff72e6499450cf, 0, 0, 0, 0x10, 2)
            0
            mstore
            invalid
            return(0,32)
        }
    }
}
