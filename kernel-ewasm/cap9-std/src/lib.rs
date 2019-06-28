#![no_std]
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate pwasm_abi;
use pwasm_abi::types::*;
use cap9_core::Serialize;

/// Generic wasm error
#[derive(Debug)]
pub struct Error;

pub mod proc_table;
pub mod syscalls;
pub use syscalls::*;

// Re-export pwasm::Vec as the Vec type for cap9_std
pub use pwasm_std::Vec;

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

pub fn write(cap_index: u8, key: &[u8; 32], value: &[u8; 32]) -> Result<(), Error> {
    let mut input = Vec::with_capacity(1 + 1 + 32 + 32);
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Write(WriteCall{key: key.into(), value: value.into()}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

pub fn call(cap_index: u8, proc_id: SysCallProcedureKey, payload: Vec<u8>) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Call(Call{proc_id: proc_id.0, payload: Payload(payload)}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

pub fn log(cap_index: u8, topics: Vec<H256>, value: Vec<u8>) -> Result<(), Error> {
    let mut input: Vec<u8> = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Log(LogCall{topics,value: Payload(value)}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

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

pub fn delete(cap_index: u8, proc_id: SysCallProcedureKey) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::Delete(DeleteProc{proc_id: proc_id.0}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}

pub fn entry(cap_index: u8, proc_id: SysCallProcedureKey) -> Result<(), Error> {
    let mut input = Vec::new();
    let syscall = SysCall {
        cap_index,
        action: SysCallAction::SetEntry(SetEntry{proc_id: proc_id.0}),
    };
    syscall.serialize(&mut input).unwrap();
    cap9_syscall(&input, &mut Vec::new())
}
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
pub trait Keyable {
    /// The width of the key in bytes.
    fn key_width() -> u8;
    fn key_slice(&self) -> Vec<u8>;
}

impl Keyable for u8 {
    fn key_width() -> u8 {
        1
    }

    fn key_slice(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(*self);
        v
    }
}

impl Keyable for Address {
    fn key_width() -> u8 {
        20
    }

    fn key_slice(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

pub trait Storable {
    /// The width of the key in bytes.
    fn store(&self) -> Vec<H256>;
    fn read(vals: Vec<H256>) -> Self;
}

impl Storable for u8 {
    fn store(&self) -> Vec<H256> {
        let u: U256 = (*self).into();
        let mut vals = Vec::new();
        vals.push(u.into());
        vals
    }

    fn read(vals: Vec<H256>) -> Self {
        let u: U256 = vals[0].into();
        u.as_u32() as u8
    }
}


impl Storable for SysCallProcedureKey {
    fn store(&self) -> Vec<H256> {
        let mut res = Vec::with_capacity(1);
        res.push(self.into());
        res
    }
    fn read(h: Vec<H256>) -> Self {
        h[0].into()
    }
}

use core::marker::PhantomData;

/// This is a Cap9 map. The way Solidity maps and Cap9 caps work are not
/// compatible, as Cap9 uses contigous storage blocks in the caps. It is
/// _generally_ expected that caps will be used in such a way that they are
/// non-overlapping (although possibly shared). This means that key-size is
/// relevant in a map that we create. This map does not do any hashing, and if a
/// hashmap is desired that should be abstracted. This map associates one key of
/// a fixed size, with a number of 32-byte values in storage.
///
/// This structure makes sense when the keys are not sparse. That is: when the
/// number of used keys is within a few orders of maginitude of the number of
/// possible keys. This will only really occur when the keys are small. A good
/// example of this is an group permission map where each group id is only
/// 8-bits.
///
/// The values of this struct are intentionally private.
///
/// The value type must implement to/from Vec<H256>.
pub struct BigMap<K,V> {
    cap_index: u8,
    /// The start location of the map.
    location: H256,
    /// The key type of the map
    key_type: PhantomData<K>,
    /// The data type of the map
    data_type: PhantomData<V>,
}


impl<K: Keyable, V: Storable> BigMap<K,V> {

    // The location is dictated by the capability. A more specific location will
    // simply require a more specific capability. This means the procedure needs
    // to access capability data.
    pub fn new(cap_index: u8) -> Self {
        // // This casts the log2 of the value size to u8. value_size is a u32, and
        // // therefore the log2 of it will always fit inside a u8. See test:
        // // log2_u32() for a demonstration of this.
        // let value_bits = f64::from(value_size).log2().ceil() as u8;
        // // Here we need to check that the capability is sufficient, otherwise we
        // // will throw an error. This will depend on the key size as well.
        //
        // The size of the cap needs to be key_width+1 in bytes
        let address_bytes = K::key_width()+1;
        let address_bits = address_bytes*8;
        let address_size = U256::from(2).pow(U256::from(address_bits));
        // The address also need to be aligned.
        let this_proc_key = proc_table::get_current_proc_id();
        if let Some(proc_table::cap::Capability::StoreWrite(proc_table::cap::StoreWriteCap {location, size})) =
                proc_table::get_proc_cap(this_proc_key, proc_table::cap::CAP_STORE_WRITE, cap_index) {
                    // Check that the size of the cap is correct.
                    if U256::from(size) < address_size {
                        panic!("cap too small")
                    } else if U256::from(location).trailing_zeros() < address_bits as u32 {
                        // the trailing number of 0 bits should be equal to or greater than the address_bits
                        panic!("cap not aligned: {}-{}", U256::from(location).trailing_zeros(), address_bits)
                    } else {
                        BigMap {
                            cap_index,
                            location: location.into(),
                            key_type: PhantomData,
                            data_type: PhantomData,
                        }
                    }
        } else {
            panic!("wrong cap: {:?}", this_proc_key)
        }
    }

    pub fn location(&self) -> H256 {
        self.location
    }

    fn base_key(&self, key: &K) -> [u8; 32] {
        let mut base: [u8; 32] = [0; 32];
        // The key start 32 - width - 1, the -1 is for data and presence
        let key_start = 32 - K::key_width() as usize - 1;
        // First we copy in the relevant parts of the location.
        base[0..key_start].copy_from_slice(&self.location().as_bytes()[0..key_start]);
        // Then we copy in the key
        // TODO: overflow
        base[key_start..(key_start+K::key_width() as usize)].clone_from_slice(key.key_slice().as_slice());
        base
    }

    fn presence_key(&self, key: &K) -> H256 {
        // The presence_key is the storage key which indicates whether there is a
        // value associated with this key.
        let mut presence_key = self.base_key(&key);
        // The first bit of the data byte indicates presence
        presence_key[31] = presence_key[31] | 0b10000000;
        presence_key.into()
    }

    pub fn present(&self, key: &K) -> bool {
        // If the value at the presence key is non-zero, then a value is
        // present.
        let present = pwasm_ethereum::read(&self.presence_key(key));
        present != [0; 32]
    }

    fn set_present(&self, key: K) {
        // If the value at the presence key is non-zero, then a value is
        // present.
        write(self.cap_index, &self.presence_key(&key).as_fixed_bytes(), H256::repeat_byte(0xff).as_fixed_bytes());
    }

    pub fn get(&self, key: K) -> Option<V> {
        // First question: Is there a value associated with this key?
        //
        // The presence_key is the storage key which indicates whether there is a
        // value associated with this key.
        let mut base = self.base_key(&key);
        if self.present(&key) {
            // TODO: remove arbitrary number
            let mut vals: Vec<H256> = Vec::with_capacity(5 as usize);
            for _ in 0..5 {
                base[31] = base[31] + 1;
                vals.push(pwasm_ethereum::read(&base.into()).into());
            }
            Some(V::read(vals))
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut base = self.base_key(&key);
        self.set_present(key);
        let vals: Vec<H256> = value.store();
        for val in vals {
            base[31] = base[31] + 1;
            write(self.cap_index, &base, &val.into());
        }
    }
}

#[cfg(test)]
mod test {
    use pwasm_abi::types::*;
    use super::*;

    #[derive(Debug,Clone,PartialEq)]
    struct ExampleData {
        key_v1: H256,
        key_v2: H256,
    }


    impl Storable for ExampleData {
        fn store(&self) -> Vec<H256> {
            let mut res = Vec::with_capacity(2);
            res.push(self.key_v1);
            res.push(self.key_v2);
            res
        }
        fn read(h: Vec<H256>) -> Self {
            ExampleData {
                key_v1: h[0],
                key_v2: h[1],
            }
        }
    }

    #[ignore]
    #[test]
    fn new_big_map() {
        let location: [u8; 32] = [
            0xaa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let proc_key: [u8; 24] = [
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
        ];
        proc_table::set_current_proc_id(proc_key).unwrap();
        let this_proc_key = proc_table::get_current_proc_id();
        let mut cap_list = Vec::new();
        cap_list.push(proc_table::cap::NewCapability {
            cap: proc_table::cap::Capability::StoreWrite(proc_table::cap::StoreWriteCap {
                location,
                size: [0xff; 32],
            }),
            parent_index: 0,
        });
        proc_table::insert_proc(this_proc_key, Address::zero(), proc_table::cap::NewCapList(cap_list)).unwrap();
        let mut map: BigMap<u8,ExampleData> = BigMap::new(0);
        assert_eq!(map.location(), location.into());
        assert_eq!(map.get(1), None);
        let example = ExampleData {
            key_v1: H256::repeat_byte(0xdd),
            key_v2: H256::repeat_byte(0xee),
        };
        map.insert(1, example.clone());
        assert_eq!(map.get(1), Some(example));
    }

    #[ignore]
    #[test]
    fn new_big_address() {
        let location: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let example_address = Address::from_slice(&[
            0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc,
            0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc,
            0xcc, 0xcc, 0xcc, 0xcc,
        ]);
        let proc_key: [u8; 24] = [
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
        ];
        proc_table::set_current_proc_id(proc_key).unwrap();
        let this_proc_key = proc_table::get_current_proc_id();
        let mut cap_list = Vec::new();
        cap_list.push(proc_table::cap::NewCapability {
            cap: proc_table::cap::Capability::StoreWrite(proc_table::cap::StoreWriteCap {
                location,
                size: [0xff; 32],
            }),
            parent_index: 0,
        });
        proc_table::insert_proc(this_proc_key, Address::zero(), proc_table::cap::NewCapList(cap_list)).unwrap();
        let mut map: BigMap<Address,ExampleData> = BigMap::new(0);
        assert_eq!(map.location(), location.into());
        assert_eq!(map.get(example_address), None);
        let example = ExampleData {
            key_v1: H256::repeat_byte(0xdd),
            key_v2: H256::repeat_byte(0xee),
        };
        map.insert(example_address, example.clone());
        assert_eq!(map.get(example_address), Some(example));
    }

    /// A sanity check to show that log2 of any u32 is less than 255, and will
    /// therefore fit inside a u8, even when rounded up.
    #[test]
    fn log2_u32() {
        let value_bits = (f64::from(u32::max_value())).log2();
        assert!(value_bits < 255_f64);
    }
}
