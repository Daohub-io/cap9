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
    use pwasm_ethereum;
    use pwasm_abi_derive::eth_abi;
    use cap9_std;

    #[eth_abi(TestWriterEndpoint, KernelClient)]
    pub trait TestWriterInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        /// Get Number
        #[constant]
        fn getNum(&mut self, key: U256) -> U256;

        fn writeNum(&mut self, cap_idx: U256, key: U256, val: U256);

    }

    pub struct WriterContract;

    impl TestWriterInterface for WriterContract {

        fn constructor(&mut self) {}

        fn getNum(&mut self, key: U256) -> U256 {
            pwasm_ethereum::read(&key.into()).into()
        }

        fn writeNum(&mut self, cap_idx: U256, key: U256, val: U256) {
            cap9_std::raw_proc_write(cap_idx.as_u32() as u8, &key.into(), &val.into()).expect("Invalid Cap Id");
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = writer::TestWriterEndpoint::new(writer::WriterContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = writer::TestWriterEndpoint::new(writer::WriterContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
