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

pub mod StorageVecTest {
    use pwasm_abi::types::*;
    use pwasm_abi_derive::eth_abi;
    use cap9_std;

    #[eth_abi(StorageVecTestEndpoint)]
    pub trait StorageVecTestInterface {

        fn constructor(&mut self);

        fn create_vector(&mut self);

        fn push_this_proc(&mut self);

        fn push_num(&mut self, num: U256);

        fn sum(&mut self) -> U256;

    }

    pub struct StorageVecTestContract;

    impl StorageVecTestInterface for StorageVecTestContract {

        fn constructor(&mut self) {}

        fn create_vector(&mut self) {
            let _vector: cap9_std::StorageVec<cap9_std::SysCallProcedureKey> = cap9_std::StorageVec::new(0);
        }

        fn push_this_proc(&mut self) {
            let mut vector: cap9_std::StorageVec<cap9_std::SysCallProcedureKey> = cap9_std::StorageVec::new(0);
            let current_proc = cap9_std::proc_table::get_current_proc_id();
            vector.push(current_proc.into());
        }

        fn push_num(&mut self, num: U256) {
            let mut vector: cap9_std::StorageVec<U256> = cap9_std::StorageVec::new(0);
            vector.push(num);
        }

        fn sum(&mut self) -> U256 {
            let vector: cap9_std::StorageVec<U256> = cap9_std::StorageVec::new(0);
            let mut total: U256 = U256::zero();
            for val in vector.iter() {
                total += val;
            }
            total
        }

    }
}
// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut endpoint = StorageVecTest::StorageVecTestEndpoint::new(StorageVecTest::StorageVecTestContract {});
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = StorageVecTest::StorageVecTestEndpoint::new(StorageVecTest::StorageVecTestContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
