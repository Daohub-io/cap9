extern crate pwasm_abi;
use pwasm_abi::types::*;
use cap9_core::Serialize;
use cap9_core::StorageValue;

/// Generic wasm error
#[derive(Debug)]
pub struct Error;

use crate::proc_table;
use crate::syscalls::*;
use crate::*;
use crate::data::*;

use core::marker::PhantomData;

/// A map of values in storage.
///
/// This is a Cap9 map. The way Solidity maps and Cap9 caps work are not
/// compatible, as Cap9 uses contiguous storage blocks in the caps. It is
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
pub struct StorageMap<K,V> {
    cap_index: u8,
    /// The start location of the map.
    location: H256,
    /// The key type of the map
    key_type: PhantomData<K>,
    /// The data type of the map
    data_type: PhantomData<V>,
}


impl<K: Keyable, V: Storable> StorageMap<K,V> {

    // The location is dictated by the capability. A more specific location will
    // simply require a more specific capability. This means the procedure needs
    // to access capability data.
    //
    // TODO: there is some risk that there is storage on this cap that is not
    // compatible. I don't know if there is a way to make this type-safe, as
    // there is no type information. Validating the current contents would be
    // prohibitively expensive.

    /// Derive a [`StorageMap`] from the cap at the given index.
    pub fn from(cap_index: u8) -> Result<Self,DataStructureError> {
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
                        Err(DataStructureError::TooSmall)
                    } else if U256::from(location).trailing_zeros() < (address_bits as u32 + 6) {
                        Err(DataStructureError::MisAligned)
                    } else {
                        Ok(StorageMap {
                            cap_index,
                            location: location.into(),
                            key_type: PhantomData,
                            data_type: PhantomData,
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

    /// Return true if the given key is associated with a value in the map.
    pub fn present(&self, key: &K) -> bool {
        // If the value at the presence key is non-zero, then a value is
        // present.
        let present = pwasm_ethereum::read(&self.presence_key(key));
        present != [0; 32]
    }

    fn set_present(&self, key: K) {
        // If the value at the presence key is non-zero, then a value is
        // present.
        write(self.cap_index, &self.presence_key(&key).as_fixed_bytes(), H256::repeat_byte(0xff).as_fixed_bytes()).unwrap();
    }

    fn set_absent(&self, key: K) {
        write(self.cap_index, &self.presence_key(&key).as_fixed_bytes(), H256::repeat_byte(0x00).as_fixed_bytes()).unwrap();
    }

    /// Get the value associated with a given key, if it exists.
    pub fn get(&self, key: K) -> Option<V> {
        let base = self.base_key(&key);
        if self.present(&key) {
            V::read(base.into())
        } else {
            None
        }
    }

    /// Insert a value at a given key.
    pub fn insert(&mut self, key: K, value: V) {
        let base = self.base_key(&key);
        self.set_present(key);
        value.store(self.cap_index, U256::from_big_endian(&base));
    }

    /// Remove a value at a given key.
    pub fn remove(&mut self, key: K) {
        let base = self.base_key(&key);
        self.set_absent(key);
        V::clear(self.cap_index, U256::from_big_endian(&base));
    }
}
