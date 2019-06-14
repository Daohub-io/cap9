#![no_std]

extern crate pwasm_abi;
use pwasm_abi::types::*;
use validator::io;
use validator::serialization::{Deserialize, Serialize};

/// Generic wasm error
#[derive(Debug)]
pub struct Error;

pub mod proc_table;

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

pub fn extcodesize(address: &Address) -> i32 {
    unsafe { external::extcodesize(address.as_ptr()) }
}

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

/// This is to replace pwasm_ethereum::call_code, and uses [`cap9_syscall_low`]: fn.cap9_syscall_low.html
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

pub fn raw_proc_write(cap_index: u8, key: &[u8; 32], value: &[u8; 32]) -> Result<(), Error> {
    let mut input = Vec::with_capacity(1 + 1 + 32 + 32);
    let syscall = SysCall::Write(key.into(), value.into());
    syscall.serialize(&mut input).unwrap();
    let mut result = Vec::with_capacity(32);
    result.resize(32,0);
    // input.resize(1+1+32+32, 0);
    cap9_syscall(&input, &mut result)
}


#[derive(Debug)]
pub enum SysCall {
    Write(U256,U256),
}

impl Deserialize for SysCall {
    type Error = io::Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let syscall_type = u8::deserialize(reader)?;
        let _cap_index = u8::deserialize(reader)?;
        match syscall_type {
            0x7 => {
                let key: U256 = U256::deserialize(reader)?;
                let value: U256 = U256::deserialize(reader)?;
                Ok(SysCall::Write(key, value))
            },
            _ => panic!("unknown syscall"),
        }
    }
}


impl Serialize for SysCall {
    type Error = io::Error;

    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            SysCall::Write(k,v) => {
                // Write syscall type
                writer.write(&[0x7])?;
                // Write cap index
                writer.write(&[0x00])?;
                // Write key
                k.serialize(writer)?;
                // Write value
                v.serialize(writer)?;
                Ok(())
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pwasm_abi::types::*;
    use validator::io;
    use validator::serialization::{Deserialize, Serialize};

    #[test]
    fn serialize_write() {
        let key: U256 = U256::zero();
        let value: U256 = U256::zero();
        let mut buffer = Vec::with_capacity(1 + 1 + 32 + 32);

        let syscall = SysCall::Write(key.into(), value.into());
        syscall.serialize(&mut buffer).unwrap();
        let expected: &[u8] = &[0x7, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00,0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00];
        assert_eq!(buffer, expected);
    }
}
