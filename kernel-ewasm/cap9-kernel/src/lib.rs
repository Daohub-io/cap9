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

use cap9_std::proc_table;
use cap9_std::*;
use cap9_std::syscalls::SysCall;

use cap9_core::{Cursor, Deserialize};

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

        fn constructor(&mut self, _entry_proc_key: String, _entry_proc_address: Address, cap_list: Vec<U256>) {
            let _entry_proc_key = {
                let byte_key = _entry_proc_key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            let cap_list = cap::NewCapList::from_u256_list(&cap_list).expect("Caplist must be valid");

            proc_table::insert_proc(_entry_proc_key, _entry_proc_address, cap_list).unwrap();
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
            let mut code = super::extcodecopy(&to);
            // TODO: FIXME: This is an awful hack. Without these two resize
            // lines (which have no net effect) we hit an Unreachable trap in
            // the WASM code. Jake's hypothesis is that these two lines trigger
            // a reallocation of some kind (of the vector) that side-steps
            // whatever issue is occuring.
            code.resize(code.len()+1,0);
            code.resize(code.len()-1,0);
            code
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
            toggle_syscall();
        }

        fn get_mode(&mut self) -> u32 {
            let current_val = pwasm_ethereum::read(&H256(super::TEST_KERNEL_SYSCALL_TOGGLE_PTR));
            current_val[0] as u32
        }

        fn panic(&mut self) {
            panic!("test-panic")
        }
    }

    pub fn toggle_syscall() {
        let mut current_val = pwasm_ethereum::read(&H256(super::TEST_KERNEL_SYSCALL_TOGGLE_PTR));
        current_val[0] = if current_val[0] == 0 { 1 } else { 0 };
        pwasm_ethereum::write(&H256(super::TEST_KERNEL_SYSCALL_TOGGLE_PTR), &current_val);
    }
}


// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn call() {
    let current_val = pwasm_ethereum::read(&H256(TEST_KERNEL_SYSCALL_TOGGLE_PTR));

    // TODO: Remove Toggling and replace current Kernel Interface with Standard Entry Procedure Interface
    //
    // If Toggled Run Entry Procedure
    // Once the entry procedure is toggled on, there is no existing mechanism to
    // turn it off.
    if current_val[0] == 1 {
        // We need to determine if this is a syscall or not. Because call_code
        // is not correctly implemented we need to think about this a little.
        // Because only delegate call is ever used, we can't use a sender
        // address.

        // If the first byte is 0x00, it is not a valid syscall. Therefore we
        // will interpret it as "toggle test mode".
        if pwasm_ethereum::input()[0] == 0x00 {
            kernel::toggle_syscall();
        }

        // If the current procedure is set to a non-zero value, we know we are
        // mid execution. Therefore this must be a system call.
        let current_proc = proc_table::get_current_proc_id();
        if current_proc == [0; 24] {
            // We are starting the kernel, therefore we should execute the entry
            // procedure.
            let proc_id: ProcedureKey = proc_table::get_entry_proc_id();
            let entry_address = proc_table::get_proc_addr(proc_id).expect("No Entry Proc");

            // Save the procedure we are about to call into "current procedure"
            proc_table::set_current_proc_id(proc_id).unwrap();
            // We need to subtract some gas from the limit, because there will
            // be instructions in-between that need to be run.
            actual_call_code(pwasm_ethereum::gas_left()-10000, &entry_address, U256::zero(), &pwasm_ethereum::input(), &mut Vec::new()).expect("Invalid Entry Proc");
            // Unset the current procedure
            proc_table::set_current_proc_id([0; 24]).unwrap();
            pwasm_ethereum::ret(&result());
        } else {
            // We are currently executing the procedure identified by
            // 'current_proc', therefore we should interpret this as a system
            // call.

            // Put the input into a cursor for deserialization.
            let input_slice = pwasm_ethereum::input();
            let mut input = Cursor::new(input_slice.as_slice());
            // Attempt to deserialize the input into a syscall. Panic on
            // deserialization failure.
            let syscall: SysCall = SysCall::deserialize(&mut input).unwrap();
            // Check the relevant cap for this procedure and the given syscall.
            let cap_ok = syscall.check_cap();
            // If the cap is ok, execute the syscall.
            if cap_ok {
                syscall.execute();
            } else {
                panic!("Bad cap");
            }
            pwasm_ethereum::ret(&result());
        }

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
    use self::pwasm_test::{ext_reset};
    use super::*;
    use core::str::FromStr;
    use kernel::KernelInterface;

    use cap9_std::proc_table::cap::*;

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

    #[ignore]
    #[test]
    fn should_parse_cap_list() {
        let prefix: u8 = 3;
        let key: [u8; 24] = [0x1,0x2,0x3,0x4,0x5,0x6,0x7,0x8,0x9,0xa,0xb,0xc,0xd,0xe,0xf,0x10,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19];
        // TODO: this need to reverse the key is an issue, but can wait
        let mut rev_key: [u8; 24] = key.clone();
        rev_key.reverse();
        let sample_cap = Capability::ProcedureCall(ProcedureCallCap {
            prefix,
            key: rev_key,
        });
        let cap_key: U256 = U256::from_big_endian(&[prefix,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x1,0x2,0x3,0x4,0x5,0x6,0x7,0x8,0x9,0xa,0xb,0xc,0xd,0xe,0xf,0x10,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19]);
        let cap_list_raw = [4.into(),3.into(),0.into(),cap_key].to_vec();
        let cap_list: NewCapList = NewCapList::from_u256_list(&cap_list_raw).unwrap();
        let expected_cap_list = NewCapList([NewCapability{parent_index:0, cap: sample_cap}].to_vec());
        assert_eq!(cap_list, expected_cap_list);
    }

}
