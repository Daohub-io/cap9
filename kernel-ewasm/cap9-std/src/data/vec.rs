
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

/// A vector of values in storage.
///
/// A key difference between a StorageVec a Rust Vec is that the
/// capacity is a property of the capability and is therefore not so flexible,
/// and cannot be changed. In order to get a vector of a different capacity you
/// need to create a new StorageVec.
pub struct StorageVec<V> {
    cap_index: u8,
    /// The start location of the map.
    location: H256,
    /// The data type of the map
    data_type: PhantomData<V>,
    /// The capacity of the vector. This is determined by the capability but is
    /// cached here on creation as it is likely to be accessed frequently, and
    /// requires multiple SREADs to determine.
    capacity: U256,
    length: U256,
}

impl<V: Storable> StorageVec<V> {

    // The location is dictated by the capability. A more specific location
    // will simply require a more specific capability. This means the procedure
    // needs to access capability data. The capacity is also defined by the
    // capability. The capability does not need to be aligned to the data size.

    /// Derive a [`StorageMap`] from the cap at the given index.
    pub fn from(cap_index: u8) -> Result<Self,DataStructureError> {
        let this_proc_key = proc_table::get_current_proc_id();
        if let Some(proc_table::cap::Capability::StoreWrite(proc_table::cap::StoreWriteCap {location, size})) =
                proc_table::get_proc_cap(this_proc_key, proc_table::cap::CAP_STORE_WRITE, cap_index) {
                    let initial_length = pwasm_ethereum::read(&H256::from(location));
                    let capacity = match (U256::from(size).saturating_add(1.into())).checked_div(V::n_keys()) {
                            // Return an error on divide-by-zero
                            None => return Err(DataStructureError::Other),
                            Some(x) => x,
                    };
                    Ok(StorageVec {
                        cap_index,
                        location: location.into(),
                        data_type: PhantomData,
                        capacity,
                        length: U256::from(initial_length),
                    })
        } else {
            Err(DataStructureError::BadCap)
        }
    }

    /// Capacity is a function of both the capability and the size of the data.
    /// We round down due to alignement.
    pub fn capacity(&self) -> U256 {
        self.capacity
    }

    pub fn length(&self) -> U256 {
        self.length
    }

    pub fn location(&self) -> H256 {
        self.location
    }

    /// Get the value at the given index.
    pub fn get(&self, index: U256) -> Option<V> {
        if index >= self.length {
            return None;
        }
        let start_key_opt: Option<U256> = match V::n_keys().checked_mul(index) {
            None => None,
            Some(x) => U256::from(self.location).checked_add(x),
        };
        let start_key: U256 = start_key_opt.unwrap();
        // We add 1 to the offset, as the first storage key us for the length of
        // the vector.
        let offset = start_key.checked_add(1.into()).unwrap();
        V::read(offset)
    }

    /// Push a value to the end of the vector.
    pub fn push(&mut self, value: V) {
        // The location of the first value in the vector (i.e. after the length
        // value).
        let start_key: U256 = U256::from(self.location).checked_add(1.into()).unwrap();
        // The offset into the vector for a new value.
        let offset: U256 = self.length.checked_mul(V::n_keys()).unwrap();
        let new_val_location: U256 = start_key.checked_add(offset).unwrap();
        value.store(self.cap_index, new_val_location);
        // Update length value.
        self.length = self.length.checked_add(1.into()).unwrap();
        // Store length value.
        write(self.cap_index, &U256::from(self.location).into(), &self.length.into()).unwrap();
    }

    /// Pop a value off the end of the vector.
    pub fn pop(&mut self) -> Option<V> {
        if self.length() < U256::from(1) {
            return None;
        }
        // The location of the first value in the vector (i.e. after the length
        // value).
        let base_key: U256 = U256::from(self.location);
        // The offset into the vector for the last value.
        let offset: U256 = self.length.checked_mul(V::n_keys()).unwrap();
        let val_location: U256 = base_key.checked_add(offset).unwrap();
        let value = V::read(val_location);
        // Clear the value from storage. Clearing away values is usually good
        // but not done on most systems as it is cheaper to overwrite it later.
        // On Ethereum we get a refund for clearing unused storage, so it is
        // actually cheaper to do so than not.
        V::clear(self.cap_index, val_location);
        // Update length value.
        self.length = self.length.checked_sub(1.into()).unwrap();
        // Store length value.
        write(self.cap_index, &U256::from(self.location).into(), &self.length.into()).unwrap();
        value
    }

    /// Produce an iterator over values in the vector.
    pub fn iter(&self) -> StorageVecIter<V> {
        StorageVecIter::new(self)
    }

}

/// An iterator over the values of a [`StorageVec`].
pub struct StorageVecIter<'a, V> {
    /// The [`StorageVec`] we are iterating over.
    storage_vec: &'a StorageVec<V>,
    /// The current offset into the [`StorageVec`].
    offset: U256,
}

impl<'a, V: Storable> StorageVecIter<'a, V> {
    fn new(storage_vec: &'a StorageVec<V>) -> Self {
        StorageVecIter {
            storage_vec,
            offset: U256::zero(),
        }
    }
}

impl<'a, V: Storable> Iterator for StorageVecIter<'a, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        match self.storage_vec.get(self.offset) {
            Some(val) => {
                self.offset += U256::from(1);
                Some(val)
            },
            None => None,
        }
    }
}
