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
    // use cap9_std::proc_table::*;
    use cap9_std::proc_table::cap::*;
    use cap9_std::syscalls::*;

    #[eth_abi(TestCallerEndpoint)]
    pub trait TestCallerInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        /// Get Number
        #[constant]
        fn testNum(&mut self) -> U256;

        // fn writeNumDirect(&mut self, key: U256, val: U256);

        fn callProc(&mut self, cap_idx: U256, key: H256, payload: Vec<u8>);

        fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256, U256);

    }

    pub struct CallerContract;

    impl TestCallerInterface for CallerContract {

        fn constructor(&mut self) {}

        fn testNum(&mut self) -> U256 {
            76.into()
        }

        fn callProc(&mut self, cap_idx: U256, key: H256, payload: Vec<u8>) {
            cap9_std::raw_proc_call(cap_idx.as_u32() as u8, key.into(), payload).unwrap();
            pwasm_ethereum::ret(&cap9_std::result());
        }

        fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256, U256) {
            // Get the key of the currently executing procedure.
            let this_key: cap9_std::proc_table::ProcedureKey = cap9_std::proc_table::get_current_proc_id();
            let cap = cap9_std::proc_table::get_proc_cap(this_key, cap_type.as_u32() as u8, cap_index.as_u32() as u8).unwrap();
            match cap {
                Capability::ProcedureCall(ProcedureCallCap {prefix, key}) => {
                    let h: H256 = SysCallProcedureKey(key).into();
                    (prefix.into(), h.into())
                },
                // ProcedureRegister(ProcedureRegisterCap),
                // ProcedureDelete(ProcedureDeleteCap),
                // ProcedureEntry(ProcedureEntryCap),
                // Capability::StoreWrite(StoreWriteCap {location, size}) => (location.into(), size.into()),
                // Log(LogCap),
                // AccountCall(AccountCallCap),
                _ => panic!("wrong cap")
            }
        }
    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = writer::TestCallerEndpoint::new(writer::CallerContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = writer::TestCallerEndpoint::new(writer::CallerContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
