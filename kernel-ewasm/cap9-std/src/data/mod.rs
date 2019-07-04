//! This module contains data structures for use with storage and capabilities.
//!
extern crate pwasm_abi;
use pwasm_abi::types::*;
use cap9_core::Serialize;
use cap9_core::StorageValue;

use crate::proc_table;
use crate::syscalls::*;
use crate::*;

use core::marker::PhantomData;

pub mod map;
pub mod map_enumerable;
pub mod vec;


#[derive(Debug)]
pub enum DataStructureError {
    /// There was an alignment error. For example, a map requires a location
    /// starting on a boundary which depends on its key size. Basically the last
    /// key_width+10 bits of the location must be zeroes.
    MisAligned,
    /// There is insufficient room in the cap. This only occurs for data
    /// structures which have a single fixed size which must be satisfied. For
    /// example, a map must be able to store every key in the key space it is
    /// designed for.
    TooSmall,
    /// The data structure was given a capability that is not of the correct
    /// type, or does not exist.
    BadCap,
    /// Miscellaneous other errors, such as divide-by-zero.
    Other,
}

// A type which implements Keyable must follow these rules:
//    1. key width must be 32 or less.
//    2. key_slice() must return a vec with a length of exactly key width.
pub trait Keyable: From<StorageValue> + Into<StorageValue> + Clone {
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

// TODO: we might be able to make this a little more typesafe, currently this is
// limited to 256 keys.
pub trait Storable: Sized {
    /// Return the number of 32-byte keys required to store a single instance of
    /// this data type.
    fn n_keys() -> U256;

    // TODO: this should use the cursor method to write directly (thereby
    // requiring a cap etc.).
    /// Convert this data into a vector of 32-byte values to be stored.
    fn store(&self, cap_index: u8, location: U256);

    // TODO: this should use the cursor method to write directly.
    /// Read an instance of this data from storage.
    fn read(location: U256) -> Option<Self>;
}

impl Storable for u8 {
    fn n_keys() -> U256 {
        1.into()
    }

    // TODO: store is 'unsafe' from a storage point of view
    fn store(&self, cap_index: u8, location: U256) {
        let u: U256 = (*self).into();
        let storage_address: H256 = H256::from(location);
        let value: H256 = u.into();
        write(cap_index, storage_address.as_fixed_bytes(), value.as_fixed_bytes()).unwrap();
    }

    fn read(location: U256) -> Option<Self> {
        let u = pwasm_ethereum::read(&location.into());
        let u: U256 = u.into();
        Some(u.as_u32() as u8)
    }
}

impl Storable for SysCallProcedureKey {

    fn n_keys() -> U256 {
        1.into()
    }

    fn store(&self, cap_index: u8, location: U256) {
        let storage_address: H256 = H256::from(location);
        let value: H256 = self.into();
        write(cap_index, storage_address.as_fixed_bytes(), value.as_fixed_bytes()).unwrap();
    }

    fn read(location: U256) -> Option<Self> {
        let h: H256 = pwasm_ethereum::read(&location.into()).into();
        Some(h.into())
    }

}

impl Storable for U256 {

    fn n_keys() -> U256 {
        1.into()
    }

    fn store(&self, cap_index: u8, location: U256) {
        let storage_address: H256 = H256::from(location);
        let value: H256 = self.into();
        write(cap_index, storage_address.as_fixed_bytes(), value.as_fixed_bytes()).unwrap();
    }

    fn read(location: U256) -> Option<Self> {
        let h: H256 = pwasm_ethereum::read(&location.into()).into();
        Some(h.into())
    }

}
