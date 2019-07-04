#![no_std]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate pwasm_abi;
use pwasm_abi::types::*;
use cap9_core::Serialize;
use cap9_core::StorageValue;

/// Procedure table.
pub mod proc_table;

/// Implementations of syscalls.
pub mod syscalls;
pub use syscalls::*;

/// Capability compatible data structures for use with Ethereum storage.
mod data;
pub use data::map::StorageMap;
use data::map::*;
pub use data::map_enumerable::StorageEnumerableMap;
use data::map_enumerable::*;
pub use data::vec::StorageVec;
use data::vec::*;

// Re-export pwasm::Vec as the Vec type for cap9_std
pub use pwasm_std::Vec;

use core::marker::PhantomData;

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

/// Low level Ethereum calls.
///
/// TODO: this is duplicated from pwasm_ethereum as it is currently in a private
/// module.
pub mod external {
    extern "C" {
        pub fn extcodesize( address: *const u8) -> i32;
        pub fn extcodecopy( dest: *mut u8, address: *const u8);
        pub fn dcall(
                gas: i64,
                address: *const u8,
                input_ptr: *const u8,
                input_len: u32,
                result_ptr: *mut u8,
                result_len: u32,
        ) -> i32;

        pub fn call_code(
                gas: i64,
                address: *const u8,
                val_ptr: *const u8,
                input_ptr: *const u8,
                input_len: u32,
                result_ptr: *mut u8,
                result_len: u32,
        ) -> i32;

        pub fn result_length() -> i32;
        pub fn fetch_result( dest: *mut u8);

        /// This extern marks an external import that we get from linking or
        /// environment. Usually this would be something pulled in from the Ethereum
        /// environement, but in this case we will use a later stage in the build
        /// process (cap9-build) to link in our own implementation of cap9_syscall
        /// to replace this import.
        ///
        /// A few notes on the API. All syscalls are delegate calls, therefore it
        /// returns an `i32` as with any other delegate call. This function here is
        /// the lowest level, therefore it's arguments are all the non-compulsory
        /// parts of a delgate call. That is, the signature of a delegate call is
        /// this:
        ///
        ///   dcall( gas: i64, address: *const u8, input_ptr: *const u8, input_len:
        ///      u32, result_ptr: *mut u8, result_len: u32, ) -> i32
        ///
        /// The `gas` and `address` are fixed by the system call specification,
        /// therefore we can only set the remaining parameters (`input_ptr`,
        /// `input_len`, `result_ptr`, and `result_len`);
        #[no_mangle]
        pub fn cap9_syscall_low(input_ptr: *const u8, input_len: u32, result_ptr: *mut u8, result_len: u32) -> i32;


    }

}

/// Return the size of the code at a given address (in bytes).
pub fn extcodesize(address: &Address) -> i32 {
    unsafe { external::extcodesize(address.as_ptr()) }
}

/// Retrieve the code at a given address.
pub fn extcodecopy(address: &Address) -> pwasm_std::Vec<u8> {
    let len = unsafe { external::extcodesize(address.as_ptr()) };
    match len {
        0 => pwasm_std::Vec::new(),
        non_zero => {
            let mut data = pwasm_std::Vec::with_capacity(non_zero as usize);
            unsafe {
                data.set_len(non_zero as usize);
                external::extcodecopy(data.as_mut_ptr(), address.as_ptr());
            }
            data
        }
    }
}

/// Performs the `CALLCODE` EVM opcode.
pub fn actual_call_code(gas: u64, address: &Address, value: U256, input: &[u8], result: &mut [u8]) -> Result<(), Error> {
    let mut value_arr = [0u8; 32];
    value.to_big_endian(&mut value_arr);
    unsafe {
        if external::call_code(
            gas as i64,
            address.as_ptr(),
            value_arr.as_ptr(),
            input.as_ptr(),
            input.len() as u32,
            result.as_mut_ptr(), result.len() as u32
        ) == 0 {
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/// Allocates and requests [`call`] return data (result)
pub fn result() -> pwasm_std::Vec<u8> {
    let len = unsafe { external::result_length() };

    match len {
        0 => pwasm_std::Vec::new(),
        non_zero => {
            let mut data = pwasm_std::Vec::with_capacity(non_zero as usize);
            unsafe {
                data.set_len(non_zero as usize);
                external::fetch_result(data.as_mut_ptr());
            }
            data
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
        external::dcall(0,0 as *const u8, 0 as *const u8, 0, 0 as *mut u8, 0);
    }
}

/// Perform a raw system call.
///
/// This is to replace pwasm_ethereum::call_code, and uses [`external::cap9_syscall_low`]: fn.cap9_syscall_low.html
/// underneath instead of dcall. This is a slightly higher level abstraction
/// over cap9_syscall_low that uses Result types and the like. This is by no
/// means part of the spec, but more ergonomic Rust level library code. Actual
/// syscalls should be built on top of this.
///
/// # Errors
///
/// Returns [`Error`] in case syscall returns error
///
/// [`Error`]: struct.Error.html
pub fn cap9_syscall(input: &[u8], result: &mut [u8]) -> Result<(), Error> {
    unsafe {
        if external::cap9_syscall_low(
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

/// Perform a write system call.
pub fn write(cap_index: u8, key: &[u8; 32], value: &[u8; 32]) -> Result<(), Error> {
    let mut input = Vec::with_capacity(1 + 1 + 32 + 32);
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Write(WriteCall{key: key.into(), value: value.into()}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

/// Perform a procedure call system call.
pub fn call(cap_index: u8, proc_id: SysCallProcedureKey, payload: Vec<u8>) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Call(Call{proc_id: proc_id.0, payload: Payload(payload)}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

/// Perform a log system call.
pub fn log(cap_index: u8, topics: Vec<H256>, value: Vec<u8>) -> Result<(), Error> {
    let mut input: Vec<u8> = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Log(LogCall{topics,value: Payload(value)}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

/// Perform a register procedure system call.
pub fn reg(cap_index: u8, proc_id: SysCallProcedureKey, address: Address, cap_list: Vec<H256>) -> Result<(), Error> {
    let mut input = Vec::new();
    let u256_list: Vec<U256> = cap_list.iter().map(|x| x.into()).collect();
    let cap_list = proc_table::cap::NewCapList::from_u256_list(&u256_list).unwrap();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Register(RegisterProc{proc_id: proc_id.0, address, cap_list}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

/// Perform a delete procedure system call.
pub fn delete(cap_index: u8, proc_id: SysCallProcedureKey) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Delete(DeleteProc{proc_id: proc_id.0}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

/// Perform a set entry system call.
pub fn entry(cap_index: u8, proc_id: SysCallProcedureKey) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::SetEntry(SetEntry{proc_id: proc_id.0}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

/// Perform an external account call system call.
pub fn acc_call(cap_index: u8, address: Address, value: U256, payload: Vec<u8>) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::AccountCall(AccountCall{
            address,
            value,
            payload: Payload(payload),
        }),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}
