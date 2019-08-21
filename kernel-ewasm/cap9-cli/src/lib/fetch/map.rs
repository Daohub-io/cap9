

use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::connection;
use crate::connection::EthConn;
use crate::project::LocalProject;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fs::File;
use std::fmt;
use cap9_std::proc_table::ProcPointer;
use cap9_std::proc_table;
use cap9_core::*;
use cap9_core::Error;
use cap9_core::Read;
use crate::constants;
use crate::utils::{from_common_u256, to_common_u256, to_common_h256,
    from_common_address, to_common_address
};
use std::collections::{HashMap, HashSet};
use cap9_std::data::{Keyable, Storable, DataStructureError};
use core::marker::PhantomData;
use super::utils::*;
use super::kernel::*;

pub struct LocalEnumerableMap<'a, 'b, T: Transport, K: Keyable, V: Storable> {
    cap_index: u8,
    /// The start location of the map.
    location: H256,
    /// The key type of the map.
    key_type: PhantomData<K>,
    /// The data type of the map.
    data_type: PhantomData<V>,
    /// Possible the cached number of elements in the map.
    length: Option<U256>,
    /// The deployed kernel used as the source of information
    kernel: &'a DeployedKernel<'a, 'b, T>,
}

impl<'a, 'b, T: Transport, K: Keyable, V: Storable> LocalEnumerableMap<'a, 'b, T, K, V> {

    pub fn from(kernel: &'a DeployedKernel<'a, 'b, T>, cap_index: u8) -> Result<Self, DataStructureError> {
        // The size of the cap needs to be key_width+1 in bytes
        let address_bytes = K::key_width()+1;
        let address_bits = address_bytes*8;
        let address_size = U256::from(2).pow(U256::from(address_bits));
        // The address also need to be aligned.

        // The cap_index is an index into the caplist of the entry procedure
        // let this_proc_key = proc_table::get_current_proc_id();
        let this_proc_key = kernel.entry_proc().0;
        // We need to get
        if let Some(proc_table::cap::Capability::StoreWrite(proc_table::cap::StoreWriteCap {location, size})) =
                kernel.get_proc_cap(this_proc_key, proc_table::cap::CAP_STORE_WRITE, cap_index) {
                    // Check that the size of the cap is correct.
                    if U256::from(size) < address_size {
                        Err(DataStructureError::TooSmall)
                    } else if U256::from(location).trailing_zeros() < (address_bits as u32 + 1 + 1 + 6) {
                        // the trailing number of 0 bits should be equal to or greater than the address_bits
                        Err(DataStructureError::MisAligned)
                    } else {
                        Ok(LocalEnumerableMap {
                            cap_index,
                            location: location.into(),
                            key_type: PhantomData,
                            data_type: PhantomData,
                            length: None,
                            kernel,
                        })
                    }
        } else {
            Err(DataStructureError::BadCap)
        }
    }

    /// Return the start/base location of the map.
    pub fn location(&self) -> H256 {
        self.location
    }

    /// Return the base storage key of a given map key.
    fn base_key(&self, key: &K) -> [u8; 32] {
        let mut base: [u8; 32] = [0; 32];
        // The key start 32 - width - 1, the - 1 is for data and presence. This
        // is in bytes.
        let key_start = 32 - K::key_width() as usize - 1;
        // First we copy in the relevant parts of the location.
        base[0..key_start].copy_from_slice(&self.location().as_bytes()[0..key_start]);
        // Then we copy in the key
        // TODO: overflow
        base[key_start..(key_start+K::key_width() as usize)].clone_from_slice(key.key_slice().as_slice());
        base
    }

    fn presence_key(&self, key: &K) -> H256 {
        // The presence_key is the storage key which indicates whether there is
        // a value associated with this key.
        let mut presence_key = self.base_key(&key);
        // The first bit of the data byte indicates presence
        presence_key[31] = presence_key[31] | 0b10000000;
        presence_key.into()
    }

    fn length_key(&self) -> H256 {
        // The presence_key is the storage key which indicates whether there is
        // a value associated with this key.
        let mut location = self.location.clone();
        let length_key = location.as_fixed_bytes_mut();
        let index = 31;
        length_key[index as usize] = length_key[index as usize] | 0b01000000;
        length_key.into()
    }

    /// Return the number of elements in the map.
    pub fn length(&self) -> U256 {
        let mut buf: [u8; 32] = [0; 32];
        h256_to_u256(self.length_key()).to_big_endian(&mut buf);
        match self.length {
            // A cached value exists, use that.
            Some(l) => l,
            // No cached value exists, read from storage.
            None => {
                let length = self.kernel.get_storage(h256_to_u256(self.length_key()));
                h256_to_u256(length)
            }
        }
    }

    // fn increment_length(&mut self) {
    //     self.length = Some(self.length().checked_add(1.into()).unwrap());
    //     // Store length value.
    //     write(self.cap_index, &self.length_key().to_fixed_bytes(), &self.length().into()).unwrap();
    // }

    // fn decrement_length(&mut self) {
    //     self.length = Some(self.length().checked_sub(1.into()).unwrap());
    //     // Store length value.
    //     write(self.cap_index, &self.length_key().to_fixed_bytes(), &self.length().into()).unwrap();
    // }

    /// Return true if the given key is associated with a value in the map.
    pub fn present(&self, key: &K) -> bool {
        // If the value at the presence key is non-zero, then a value is
        // present.
        let presence_key = h256_to_u256(self.presence_key(key));
        let mut buf = [0;32];
        presence_key.to_big_endian(&mut buf);
        let present = self.kernel.get_storage(presence_key);
        let null: [u8; 32] = [0; 32];
        present.as_fixed_bytes() != &null
    }

    // fn index(&self, key: &K) -> Option<U256> {
    //     let present = pwasm_ethereum::read(&self.presence_key(key));
    //     Some(present.into())
    // }

    // fn set_present(&self, key: &K, index: U256) {
    //     // For the enumerable map, the presence value is a 1-based index into
    //     // the enumeration vector.
    //     let storable_index: StorageValue = index.into();
    //     write(self.cap_index, &self.presence_key(key).as_fixed_bytes(), &storable_index.into()).unwrap();
    // }

    // fn set_absent(&self, key: K) {
    //     write(self.cap_index, &self.presence_key(&key).as_fixed_bytes(), H256::repeat_byte(0x00).as_fixed_bytes()).unwrap();
    // }

    // /// Get the value associated with a given key, if it exists.
    pub fn get(&self, key: K) -> Option<V> {
        let base = self.base_key(&key);
        if self.present(&key) {
            let mut vals = Vec::new();
            let base_address: U256 = base.into();
            for i in 0..V::n_keys().as_u64() {
                let val = self.kernel.get_storage(base_address + U256::from(i));
                vals.push(to_common_u256(h256_to_u256(val)));
            }
            V::read_vec_u256(vals)
        } else {
            None
        }
    }

    /// Return the key at a given index in the map. The ordering of keys is not
    /// well defined, and this should only be used for enumeration.
    pub fn get_key_at_index(&self, index: U256) -> Option<K> {
        if index >= self.length() {
            return None;
        }
        let storage_key_h = self.length_key().clone();
        let storage_key: U256 = h256_to_u256(storage_key_h) + index + U256::from(1);
        let mut store_buf: [u8; 32] = [0; 32];
        storage_key.to_big_endian(&mut store_buf);
        let storage_value: StorageValue = to_common_h256(self.kernel.get_storage(storage_key)).into();
        Some(storage_value.into())
    }

    /// Produce an iterator over keys and values.
    pub fn iter(&self) -> LocalEnumerableMapIter<T, K, V> {
        LocalEnumerableMapIter::new(self)
    }

    // /// Produce an iterator over keys.
    // pub fn keys(&self) -> StorageEnumerableMapKeys<K,V> {
    //     StorageEnumerableMapKeys::new(self)
    // }

    // /// Produce an iterator over values.
    // pub fn values(&self) -> StorageEnumerableMapValues<K,V> {
    //     StorageEnumerableMapValues::new(self)
    // }
}


/// An iterator over the keys and values of a [`StorageEnumerableMap`].
pub struct LocalEnumerableMapIter<'a, 'b, 'c, T: Transport, K: Keyable, V: Storable> {
    /// The StorageVec we are iterating over.
    storage_map: &'a LocalEnumerableMap<'b, 'c, T, K, V>,
    /// The current offset into the StorageVec.
    offset: U256,
}

impl<'a, 'b, 'c, T: Transport, K: Keyable, V: Storable> LocalEnumerableMapIter<'a, 'b, 'c, T, K, V> {
    fn new(storage_map: &'a LocalEnumerableMap<'b, 'c, T, K, V>) -> Self {
        LocalEnumerableMapIter {
            storage_map,
            offset: U256::zero(),
        }
    }
}

impl<'a, 'b, 'c, T: Transport, K: Keyable, V: Storable> Iterator for LocalEnumerableMapIter<'a, 'b, 'c, T, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let key = match self.storage_map.get_key_at_index(self.offset) {
            Some(val) => {
                self.offset += U256::from(1);
                val
            },
            None => {
                return None;
            },
        };
        Some((key.clone(), self.storage_map.get(key)?))
    }
}
