#![no_std]
#![allow(non_snake_case)]

extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_ethereum;
extern crate parity_wasm;
extern crate validator;
extern crate cap9_std;

use cap9_std::*;

// When we are compiling to WASM, unresolved references are left as (import)
// expressions. However, under any other target symbols will have to be linked
// for EVM functions (blocknumber, create, etc.). Therefore, when we are not
// compiling for WASM (be it test, realse, whatever) we want to link in dummy
// functions. pwasm_test provides all the builtins provided by parity, while
// cap9_test covers the few that we have implemented ourselves.

fn main() {}

pub mod entry {
    use pwasm_abi::types::*;
    use pwasm_ethereum;
    use pwasm_abi_derive::eth_abi;
    use cap9_std;

    use validator::{Validity, Module};

    #[eth_abi(TestValidatorEndpoint)]
    pub trait TestValidatorInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        /// Get Number
        #[constant]
        fn getNum(&mut self) -> U256;

        fn callExternal(&mut self, cap_idx: U256, address: Address, value: U256, payload: Vec<u8>);

        /// Check if Procedure Contract is Valid
        fn check_contract(&mut self, _to: Address) -> bool;
        /// Get the size (in bytes) of another contract
        fn get_code_size(&mut self, _to: Address) -> i32;
        /// Copy the code of another contract into memory
        fn code_copy(&mut self, _to: Address) -> pwasm_std::Vec<u8>;
    }

    pub struct ValidatorContract;

    impl TestValidatorInterface for ValidatorContract {

        fn constructor(&mut self) {}

        fn getNum(&mut self) -> U256 {
            U256::from(6)
        }

        fn callExternal(&mut self, cap_idx: U256, address: Address, value: U256, payload: Vec<u8>) {
            cap9_std::acc_call(cap_idx.as_u32() as u8, address, value, payload).unwrap();
            pwasm_ethereum::ret(&cap9_std::result());
        }

        fn check_contract(&mut self, target: Address) -> bool {
            // First we check if the target is the null address. If so we return
            // false.
            if target == H160::zero() {
                false
            } else {
                // Next we get the code of the contract, using EXTCODECOPY under
                // the hood.
                let code: pwasm_std::Vec<u8> = self.code_copy(target);
                Module::new(code.as_slice()).is_valid()
            }
        }

        fn get_code_size(&mut self, to: Address) -> i32 {
            super::extcodesize(&to)
        }

        fn code_copy(&mut self, to: Address) -> pwasm_std::Vec<u8> {
            let mut code = super::extcodecopy(&to);
            // TODO: FIXME: This is an awful hack. Without these two resize
            // lines (which have no net effect) we hit an Unreachable trap in
            // the WASM code. Jake's hypothesis is that these two lines trigger
            // a reallocation of some kind (of the vector) that side-steps
            // whatever issue is occuring.
            code.resize(code.len()-1,0);
            code.resize(code.len()+1,0);
            code
        }

    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = entry::TestValidatorEndpoint::new(entry::ValidatorContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = entry::TestValidatorEndpoint::new(entry::ValidatorContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
