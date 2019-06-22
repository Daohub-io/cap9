#![no_std]
#![allow(non_snake_case)]

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

    #[eth_abi(TestAccountCallEndpoint)]
    pub trait TestAccountCallInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        /// Get Number
        #[constant]
        fn getNum(&mut self) -> U256;

        fn callExternal(&mut self, cap_idx: U256, address: Address, value: U256, payload: Vec<u8>);

    }

    pub struct AccountCallContract;

    impl TestAccountCallInterface for AccountCallContract {

        fn constructor(&mut self) {}

        fn getNum(&mut self) -> U256 {
            U256::from(6)
        }

        fn callExternal(&mut self, cap_idx: U256, address: Address, value: U256, payload: Vec<u8>) {
            cap9_std::raw_proc_acc_call(cap_idx.as_u32() as u8, address, value, payload).unwrap();
            pwasm_ethereum::ret(&cap9_std::result());
        }

    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = entry::TestAccountCallEndpoint::new(entry::AccountCallContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = entry::TestAccountCallEndpoint::new(entry::AccountCallContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
