#![allow(non_snake_case)]
#![feature(proc_macro_hygiene)]
#![no_std]

extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_ethereum;
extern crate parity_wasm;
extern crate validator;
extern crate cap9_std;

use pwasm_abi::types::*;
use core::default::Default;

use cap9_std::proc_table;
use cap9_std::*;

/// This is a temporary storage location for toggling
const TEST_KERNEL_SYSCALL_TOGGLE_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

type ProcedureKey = [u8; 24];

pub mod kernel {
    use pwasm_abi::types::*;
    use validator::{Validity, Module};
    use pwasm_ethereum;

    use crate::proc_table;
    use crate::proc_table::cap;

    use pwasm_abi_derive::eth_abi;

    #[eth_abi(TestKernelEndpoint, KernelClient)]
    pub trait KernelInterface {
        /// The constructor set with Initial Entry Procedure
        fn constructor(&mut self, _entry_proc_key: String, _entry_proc_address: Address, _cap_list: Vec<U256>);
        /// Get Entry Procedure
        #[constant]
        fn entryProcedure(&mut self) -> [u8; 24];
        /// Get Current Executing Procedure
        #[constant]
        fn currentProcedure(&mut self) -> [u8; 24];

        /// Get Procedure Address By Key
        /// Returns 0 if Procedure Not Found
        fn getProcedureByKey(&mut self, _proc_key: String) -> Address;
        /// Check if Procedure Contract is Valid
        fn check_contract(&mut self, _to: Address) -> bool;
        /// Get the size (in bytes) of another contract
        fn get_code_size(&mut self, _to: Address) -> i32;
        /// Copy the code of another contract into memory
        fn code_copy(&mut self, _to: Address) -> pwasm_std::Vec<u8>;

        /// Get Cap Length by Type
        fn get_cap_type_len(&mut self, _proc_key: String, _cap_type: U256) -> U256;

        /// Toggle Syscall Mode
        /// (Forwards all calls to entry procedure)
        ///
        /// _Temporary for debugging purposes_
        fn toggle_syscall(&mut self);

        fn get_mode(&mut self) -> u32;

        fn panic(&mut self);
    }

    pub struct KernelContract;

    impl KernelInterface for KernelContract {

        fn constructor(&mut self, _entry_proc_key: String, _entry_proc_address: Address, _cap_list: Vec<U256>) {
            let _entry_proc_key = {
                let byte_key = _entry_proc_key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            let _cap_list = cap::NewCapList::from_u256_list(&_cap_list).expect("Caplist must be valid");

            proc_table::insert_proc(_entry_proc_key, _entry_proc_address, _cap_list).unwrap();
            proc_table::set_entry_proc_id(_entry_proc_key).unwrap();
        }

        fn entryProcedure(&mut self) -> [u8; 24] {
            proc_table::get_entry_proc_id()
        }

        fn currentProcedure(&mut self) -> [u8; 24] {
            proc_table::get_current_proc_id()
        }

        fn getProcedureByKey(&mut self, _proc_key: String) -> Address {
            let _proc_key = {
                let byte_key = _proc_key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            proc_table::get_proc_addr(_proc_key).unwrap_or(H160::zero())
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
            super::extcodecopy(&to)
        }

        fn get_cap_type_len(&mut self, _proc_key: String, _cap_type: U256) -> U256 {
            let _proc_key = {
                let byte_key = _proc_key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            U256::from(proc_table::get_proc_cap_list_len(_proc_key, _cap_type.as_u32() as u8))
        }

        fn toggle_syscall(&mut self) {
            let mut current_val = pwasm_ethereum::read(&H256(super::TEST_KERNEL_SYSCALL_TOGGLE_PTR));
            current_val[0] = if current_val[0] == 0 { 1 } else { 0 };
            pwasm_ethereum::write(&H256(super::TEST_KERNEL_SYSCALL_TOGGLE_PTR), &current_val);
        }

        fn get_mode(&mut self) -> u32 {
            let current_val = pwasm_ethereum::read(&H256(super::TEST_KERNEL_SYSCALL_TOGGLE_PTR));
            current_val[0] as u32
        }

        fn panic(&mut self) {
            panic!("test-panic")
        }
    }
}

// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let mut current_val = pwasm_ethereum::read(&H256(TEST_KERNEL_SYSCALL_TOGGLE_PTR));

    // TODO: Remove Toggling and replace current Kernel Interface with Standard Entry Procedure Interface
    //
    // If Toggled Run Entry Procedure
    // Once the entry procedure is toggled on, there is no existing mechanism to
    // turn it off.
    if current_val[0] == 1 {
        let entry_address = proc_table::get_proc_addr(proc_table::get_entry_proc_id()).expect("No Entry Proc");

        // TODO:
        // We don't have `returnDataSize` or `returnDataCopy`, https://github.com/ewasm/design/blob/master/eth_interface.md#getreturndatasize
        // So we'll simply allocate a large result buffer for now
        let mut result = [6; 32].to_vec();
        // NB: This "call_code" is a DELEGATECALL not a CALLCODE.
        pwasm_ethereum::call_code(1000, &entry_address, &pwasm_ethereum::input(), &mut result).expect("Invalid Entry Proc");
        pwasm_ethereum::ret(&result);

    } else {
        // If not toggled expose test interface
        let mut endpoint = kernel::TestKernelEndpoint::new(kernel::KernelContract {});
        // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
        pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
    }
}

#[no_mangle]
pub fn deploy() {
    let mut endpoint = kernel::TestKernelEndpoint::new(kernel::KernelContract {});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    use self::pwasm_test::{ext_get, ext_reset};
    use super::*;
    use core::str::FromStr;
    use pwasm_abi::types::*;
    use kernel::KernelInterface;

    #[test]
    fn should_initialize_with_entry_procedure() {
        let mut contract = kernel::KernelContract {};

        let owner_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let entry_proc_key = pwasm_abi::types::String::from("init");
        let entry_proc_address =
            Address::from_str("db6fd484cfa46eeeb73c71edee823e4812f9e2e1").unwrap();

        // Here we're creating an External context using ExternalBuilder and set the `sender` to the `owner_address`
        // so `pwasm_ethereum::sender()` in KernelInterface::constructor() will return that `owner_address`
        ext_reset(|e| e.sender(owner_address.clone()));

        contract.constructor(entry_proc_key.clone(), entry_proc_address.clone(), Vec::new());

        assert_eq!(&contract.entryProcedure()[0..4], entry_proc_key.as_bytes());
        assert_eq!(contract.currentProcedure(), [0u8; 24]);
    }

}
