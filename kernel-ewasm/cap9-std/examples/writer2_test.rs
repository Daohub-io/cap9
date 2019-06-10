#![cfg_attr(not(feature="std"), no_std)]
#![no_main]
#![allow(non_snake_case)]

extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;

use pwasm_abi::types::*;
use pwasm_abi_derive::eth_abi;
use pwasm_ethereum::Error;

fn main() {}

/// TODO: this is duplicated from pwasm_ethereum as it is currently in a private
/// module.
extern "C" {
    pub fn dcall(
            gas: i64,
            address: *const u8,
            input_ptr: *const u8,
            input_len: u32,
            result_ptr: *mut u8,
            result_len: u32,
) -> i32;
}

extern "C" {
    /// This extern marks an external import that we get from linking or
    /// environment. Usually this would be something pulled in from the Ethereum
    /// environement, but in this case we will use a later stage in the build
    /// process (cap9-build) to link in our own implementation of cap9_syscall
    /// to replace this import.
    ///
    /// A few notes on the API. All syscalls are delegate calls, therefore it
    /// returns an i32 as with any other delegate call. This function here is
    /// the lowest level, therefore it's arguments are all the non-compulsory
    /// parts of a delgate call. That is, the signature of a delegate call is
    /// this:
    ///
    ///   dcall( gas: i64, address: *const u8, input_ptr: *const u8, input_len:
    ///      u32, result_ptr: *mut u8, result_len: u32, ) -> i32
    ///
    /// But gas and address are fixed by the system call specification,
    /// therefore we can only set the remaining parameters (input_ptr,
    /// input_len, result_ptr, and result_len);
    #[no_mangle]
    pub fn cap9_syscall_low(input_ptr: *const u8, input_len: u32, result_ptr: *mut u8, result_len: u32, ) -> i32;
}

/// This is to replace pwasm_ethereum::call_code, and uses cap9_syscall_low
/// underneath instead of dcall. This is a slightly higher level abstraction
/// over cap9_syscall_low that uses Result types and the like. This is by no
/// means part of the spec, but more ergonomic Rust level library code. Actual
/// syscalls should be built on top of this.
pub fn cap9_syscall(input: &[u8], result: &mut [u8]) -> Result<(), Error> {
    unsafe {
        if cap9_syscall_low(
            input.as_ptr(),
            input.len() as u32,
            result.as_mut_ptr(),
            result.len() as u32
        ) == 0 {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/// This function is the rough shape of a syscall. It's only purpose is to force
/// the inclusion/import of all the necessay Ethereum functions and prevent them
/// from being deadcode eliminated. As part of this, it is also necessary to
/// pass wasm-build "dummy_syscall" as a public api parameter, to ensure that it
/// is preserved.
///
/// TODO: this is something we would like to not have to do
#[no_mangle]
fn dummy_syscall() {
    pwasm_ethereum::gas_left();
    pwasm_ethereum::sender();
    unsafe {
        dcall(0,0 as *const u8, 0 as *const u8, 0, 0 as *mut u8, 0);
    }
}

/// The call function is the main function of the *deployed* contract
#[no_mangle]
pub fn call() {
    // write some value
    pwasm_ethereum::write(&pwasm_std::types::H256::zero(), &[0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1]);
    // call another contract
    pwasm_ethereum::call(0, &Address::zero(), pwasm_std::types::U256::zero(), &[], &mut [] );
    // delegate call another contract (under the hood this version of call_code
    // uses delgate call).
    pwasm_ethereum::gas_left();
    pwasm_ethereum::sender();

    // An example syscall (empty input and output)
    cap9_syscall(&[], &mut []);

    pwasm_ethereum::ret(&b"result"[..]);
    let mut endpoint = contract::ExampleContract1Endpoint::new(contract::ExampleContract1{});
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

// Declares the dispatch and dispatch_ctor methods
use pwasm_abi::eth::EndpointInterface;

#[no_mangle]
pub fn deploy() {
    let mut endpoint = contract::ExampleContract1Endpoint::new(contract::ExampleContract1{});
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}


pub mod contract {
    use super::*;
    use pwasm_abi_derive::eth_abi;

    #[eth_abi(ExampleContract1Endpoint, ExampleContract1Client)]
    pub trait ExampleContract1Interface {
        /// Check if Procedure Contract is Valid
        fn check_contract(&mut self, _to: Address) -> bool;
    }

    pub struct ExampleContract1;

    impl ExampleContract1Interface for ExampleContract1 {
        fn check_contract(&mut self, _target: Address) -> bool {
            // unimplemented!()
            false
        }
    }
}
