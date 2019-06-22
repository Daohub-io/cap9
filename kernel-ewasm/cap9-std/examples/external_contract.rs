#![no_std]
#![allow(non_snake_case)]

extern crate cap9_std;
extern crate pwasm_std;
extern crate pwasm_abi_derive;

// When we are compiling to WASM, unresolved references are left as (import)
// expressions. However, under any other target symbols will have to be linked
// for EVM functions (blocknumber, create, etc.). Therefore, when we are not
// compiling for WASM (be it test, realse, whatever) we want to link in dummy
// functions. pwasm_test provides all the builtins provided by parity, while
// cap9_test covers the few that we have implemented ourselves.
#[cfg(not(target_arch = "wasm32"))]
extern crate pwasm_test;
#[cfg(not(target_arch = "wasm32"))]
extern crate cap9_test;

fn main() {}

pub mod writer {
    use pwasm_abi::types::*;
    use pwasm_abi_derive::eth_abi;

    #[eth_abi(TestExternalEndpoint, KernelClient)]
    pub trait TestExternalInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        /// Get Number
        #[payable]
        fn testNum(&mut self) -> U256;

    }

    pub struct ExternalContract;

    impl TestExternalInterface for ExternalContract {

        fn constructor(&mut self) {}

        fn testNum(&mut self) -> U256 {
            56.into()
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = writer::TestExternalEndpoint::new(writer::ExternalContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = writer::TestExternalEndpoint::new(writer::ExternalContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
