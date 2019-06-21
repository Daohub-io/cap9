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

    #[eth_abi(TestWriterEndpoint, KernelClient)]
    pub trait TestWriterInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self);

        /// Get Number
        #[constant]
        fn getNum(&mut self, key: U256) -> U256;

        fn writeNumDirect(&mut self, key: U256, val: U256);

        fn writeNum(&mut self, cap_idx: U256, key: U256, val: U256);

        fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256, U256);

        fn getEntry(&mut self) -> H256;

    }

    pub struct WriterContract;

    impl TestWriterInterface for WriterContract {

        fn constructor(&mut self) {}

        fn getNum(&mut self, key: U256) -> U256 {
            pwasm_ethereum::read(&key.into()).into()
        }

        // Write to storage without going through the kernel or cap system
        fn writeNumDirect(&mut self, key: U256, val: U256) {
            pwasm_ethereum::write(&key.into(), &val.into());
        }

        fn writeNum(&mut self, cap_idx: U256, key: U256, val: U256) {
            cap9_std::raw_proc_write(cap_idx.as_u32() as u8, &key.into(), &val.into()).unwrap();
        }

        fn getCap(&mut self, cap_type: U256, cap_index: U256) -> (U256, U256) {
            // Get the key of the currently executing procedure.
            let this_key: cap9_std::proc_table::ProcedureKey = cap9_std::proc_table::get_current_proc_id();
            let cap = cap9_std::proc_table::get_proc_cap(this_key, cap_type.as_u32() as u8, cap_index.as_u32() as u8).unwrap();
            match cap {
                // ProcedureCall(ProcedureCallCap),
                // ProcedureRegister(ProcedureRegisterCap),
                // ProcedureDelete(ProcedureDeleteCap),
                // ProcedureEntry(ProcedureEntryCap),
                Capability::StoreWrite(StoreWriteCap {location, size}) => (location.into(), size.into()),
                // Log(LogCap),
                // AccountCall(AccountCallCap),
                _ => panic!("wrong cap")
            }
        }

        fn getEntry(&mut self) -> H256 {
            let proc_id = cap9_std::proc_table::get_entry_proc_id();
            cap9_std::syscalls::SysCallProcedureKey(proc_id).into()
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
