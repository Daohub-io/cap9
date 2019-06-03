extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_ethereum;
extern crate pwasm_std;

use pwasm_abi::eth;
use pwasm_abi::types::*;
use pwasm_abi_derive::eth_abi;

const KERNEL_PROC_HEAP_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];
const KERNEL_PROC_LIST_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];
const KERNEL_ADDRESS_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];
const KERNEL_CURRENT_PROC_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];
const KERNEL_CURRENT_ENTRY_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

type ProcedureKey = [u8; 24];
type ProcedureIndex = [u8; 24];

pub mod cap;

pub struct ProcPointer(ProcedureKey);

impl ProcPointer {
    fn from_key(key: ProcedureKey) -> ProcPointer {
        ProcPointer(key)
    }

    fn get_store_ptr(&self) -> [u8; 32] {
        let mut result: [u8; 32] = KERNEL_PROC_HEAP_PTR;
        result[5..29].copy_from_slice(&self.0);
        result
    }

    fn get_addr_ptr(&self) -> [u8; 32] {
        self.get_store_ptr()
    }

    fn get_index_ptr(&self) -> [u8; 32] {
        let mut pointer = self.get_store_ptr();
        pointer[31] = 1;
        pointer
    }

    fn get_cap_type_len_ptr(&self, cap_type: u8) -> [u8; 32] {
        let mut pointer = self.get_store_ptr();
        pointer[29] = 1;
        pointer
    }

    fn get_list_ptr(index: U256) -> [u8; 32] {
        let mut result: [u8; 32] = KERNEL_PROC_LIST_PTR;
        let slice: [u8; 32] = index.into();
        result[5..29].copy_from_slice(&slice[8..]);
        result
    }
}

/// Error or Procedure Insertion
pub enum ProcInsertError {
    /// Procedure Id Already Used
    UsedId = 2,
    /// Procedure List length is greater than 255
    ListFull = 3,
}

/// Inserts Procedure into procedure table
pub fn insert_proc(key: ProcedureKey, address: Address, cap_list: cap::NewCapList) -> Result<(), ProcInsertError> {
    // Get Procedure Storage
    let proc_pointer = ProcPointer::from_key(key);

    // Check Procedure Index
    // If Index Is Greater than zero the procedure already exists
    let proc_index = pwasm_ethereum::read(&H256(proc_pointer.get_index_ptr()));
    if proc_index[31] != 0 {
        return Err(ProcInsertError::UsedId);
    }

    // We assign this procedure then next key index
    // Check Procedure List Length, it must be less than 8^24
    let proc_list_len = pwasm_ethereum::read(&H256(KERNEL_PROC_LIST_PTR));
    let new_proc_index = U256::from(proc_list_len) + 1;
    if new_proc_index.leading_zeros() < 8 {
        return Err(ProcInsertError::ListFull);
    }

    // Store Address
    pwasm_ethereum::write(
        &H256(proc_pointer.get_addr_ptr()),
        H256::from(address).as_fixed_bytes(),
    );

    // Store Index
    pwasm_ethereum::write(&H256(proc_pointer.get_index_ptr()), &new_proc_index.into());

    // Store Key
    let mut key_input = [0; 32];
    key_input[8..].copy_from_slice(&key);

    pwasm_ethereum::write(&H256(ProcPointer::get_list_ptr(new_proc_index)), &key_input);

    // Update Proc List Len
    pwasm_ethereum::write(&H256(KERNEL_PROC_LIST_PTR), &new_proc_index.into());

    // TODO: Store CapList
    // if cap_list.0.len() > 0 {
    //     unimplemented!();
    // }

    Ok(())
}

/// Error on Procedure Removal
pub enum ProcRemoveError {
    /// Procedure Id is not Used
    InvalidId = 2,
    /// Procedure is the Entry Procedure which cannot be removed
    EntryProc = 3,
}
pub fn remove_proc(key: ProcedureKey) -> Result<(), ProcRemoveError> {
    // Get Procedure Storage
    let proc_pointer = ProcPointer::from_key(key);

    // Check Procedure Index
    // If Index Is Greater than zero the procedure already exists
    let proc_index = pwasm_ethereum::read(&H256(proc_pointer.get_index_ptr()));
    if proc_index[31] == 0 {
        return Err(ProcRemoveError::InvalidId);
    }

    // Check Procedure is not the Entry Procedure
    let entry_id = get_entry_proc_id();
    if entry_id == key {
        return Err(ProcRemoveError::EntryProc);
    }

    // Check Procedure List Length, it must be greater than 1;
    let proc_list_len = pwasm_ethereum::read(&H256(KERNEL_PROC_LIST_PTR));
    assert!(U256::from(proc_list_len) >= U256::one());

    // If Removed Procedure Is not the last
    // Overwrite the removed procedure key in the list with the last on
    if proc_index != proc_list_len {
        let last_proc_id =
            pwasm_ethereum::read(&H256(ProcPointer::get_list_ptr(U256::from(proc_list_len))));
        pwasm_ethereum::write(
            &H256(ProcPointer::get_list_ptr(U256::from(proc_index))),
            &last_proc_id,
        );
    }

    // Decrement Proc List Len
    let new_proc_index = U256::from(proc_list_len) - 1;
    pwasm_ethereum::write(&H256(KERNEL_PROC_LIST_PTR), &new_proc_index.into());

    // Remove Last Proc Id From List
    pwasm_ethereum::write(
        &H256(ProcPointer::get_list_ptr(U256::from(proc_list_len))),
        &[0; 32],
    );

    // Remove Address
    pwasm_ethereum::write(&H256(proc_pointer.get_addr_ptr()), &[0; 32]);

    // Remove Index
    pwasm_ethereum::write(&H256(proc_pointer.get_index_ptr()), &[0; 32]);

    // Todo: Remove CapList
    Ok(())
}

fn contains(key: ProcedureKey) -> bool {
    // Get Procedure Storage
    let proc_pointer = ProcPointer::from_key(key);

    // Check Procedure Index
    // If Index Is Greater than zero the procedure already exists
    let proc_index = pwasm_ethereum::read(&H256(proc_pointer.get_index_ptr()));
    proc_index[31] != 0
}

/// Get Procedure Address By Key
fn get_proc_addr(key: ProcedureKey) -> Option<Address> {
    // Get Procedure Storage
    let proc_pointer = ProcPointer::from_key(key);
    let proc_addr = pwasm_ethereum::read(&H256(proc_pointer.get_addr_ptr()));

    // Check if Address is Zero
    if proc_addr == [0; 32] {
        None
    } else {
        Some(H256(proc_addr).into())
    }
}

/// Get Procedure Index By Key
fn get_proc_index(key: ProcedureKey) -> Option<ProcedureIndex> {
    // Get Procedure Storage
    let proc_pointer = ProcPointer::from_key(key);
    let proc_index = pwasm_ethereum::read(&H256(proc_pointer.get_index_ptr()));

    if proc_index == [0; 32] {
        None
    } else {
        let mut result = [0; 24];
        result.copy_from_slice(&proc_index[8..]);
        Some(result)
    }
}

/// Get Procedure Key By Index
fn get_proc_id(index: ProcedureIndex) -> Option<ProcedureKey> {
    let index = {
        let mut output = [0u8; 32];
        output[8..].copy_from_slice(&index);
        U256::from(output)
    };

    let proc_id = pwasm_ethereum::read(&H256(ProcPointer::get_list_ptr(index)));

    if proc_id == [0; 32] {
        None
    } else {
        let mut result = [0; 24];
        result.copy_from_slice(&proc_id[8..]);
        Some(result)
    }
}

/// Get Procedure List Length
fn get_proc_list_len() -> U256 {
    // Check Procedure List Length, it must be less than 8^24
    let proc_list_len = pwasm_ethereum::read(&H256(KERNEL_PROC_LIST_PTR));
    U256::from(proc_list_len)
}

/// Get Entry Procedure Id
fn get_entry_proc_id() -> ProcedureKey {
    let proc_id = pwasm_ethereum::read(&H256(KERNEL_CURRENT_ENTRY_PTR));
    let mut result = [0; 24];
    result.copy_from_slice(&proc_id[8..]);
    result
}

#[cfg(test)]
pub mod contract {
    use super::*;

    #[eth_abi(ProcedureEndpoint, ProcedureClient)]
    pub trait ProcedureTableInterface {
        /// Insert Procedure By Key
        fn insert_proc(&mut self, key: String, address: Address, cap_list: Vec<U256>) -> U256;

        /// Remove Procedure By Key
        fn remove_proc(&mut self, key: String) -> U256;

        /// Check if Procedure Exists By Key
        fn contains(&mut self, key: String) -> bool;

        /// Get Procedure List Length
        fn get_proc_list_len(&mut self) -> U256;

        /// Get Procedure Address By Key
        fn get_proc_addr(&mut self, key: String) -> Address;

        /// Get Procedure Index By Key
        fn get_proc_index(&mut self, key: String) -> U256;

        /// Get Procedure Key By Index
        fn get_proc_id(&mut self, index: U256) -> String;

        /// Get Procedure Cap List Length By Id and Type
        fn get_proc_cap_list_len(&mut self, key: String, cap_type: U256) -> U256;

        /// Get Procedure Capability by Id, Type and Index
        fn get_proc_cap(&mut self, key: String, cap_type: U256, cap_index: U256) -> Vec<U256>;

        /// Get Entry Procedure Id
        fn get_entry_proc_id(&mut self) -> String;
    }

    pub struct ProcedureTableContract;

    impl ProcedureTableInterface for ProcedureTableContract {
        fn insert_proc(&mut self, key: String, address: Address, cap_list: Vec<U256>) -> U256 {
            let raw_key = {
                let mut byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            let decoded_cap_list = cap::NewCapList::from_u256_list(&cap_list).expect("Caplist should be valid");

            match insert_proc(raw_key, address, decoded_cap_list) {
                Ok(()) => U256::zero(),
                Err(_) => U256::one(),
            }
        }

        fn remove_proc(&mut self, key: String) -> U256 {
            let raw_key = {
                let mut byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            match remove_proc(raw_key) {
                Ok(()) => U256::zero(),
                Err(_) => U256::one(),
            }
        }

        fn contains(&mut self, key: String) -> bool {
            let raw_key = {
                let mut byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            contains(raw_key)
        }

        fn get_proc_list_len(&mut self) -> U256 {
            get_proc_list_len()
        }

        fn get_proc_addr(&mut self, key: String) -> Address {
            let raw_key = {
                let mut byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            if let Some(addr) = get_proc_addr(raw_key) {
                addr
            } else {
                H160::zero()
            }
        }

        fn get_proc_index(&mut self, key: String) -> U256 {
            let raw_key = {
                let mut byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            if let Some(index) = get_proc_index(raw_key) {
                let mut output = [0u8; 32];
                output[8..].copy_from_slice(&index);
                U256::from(output)
            } else {
                U256::zero()
            }
        }

        fn get_proc_id(&mut self, index: U256) -> String {
            let raw_index = {
                let mut output = [0u8; 24];
                let temp: [u8; 32] = index.into();
                output.copy_from_slice(&temp[8..]);
                output
            };

            if let Some(id) = get_proc_id(raw_index) {
                unsafe { String::from_utf8_unchecked(id.to_vec()) }
            } else {
                String::new()
            }
        }

        fn get_proc_cap_list_len(&mut self, key: String, cap_type: U256) -> U256 {
            unimplemented!()
        }

        fn get_proc_cap(&mut self, key: String, cap_type: U256, cap_index: U256) -> Vec<U256> {
            unimplemented!()
        }

        fn get_entry_proc_id(&mut self) -> String {
            unimplemented!()
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;
    extern crate std;

    use super::contract;
    use super::contract::*;
    use super::cap::*;

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    #[test]
    fn should_insert_proc_by_key() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        
        let sample_cap = StoreWriteCap {
            location: U256::from(1234).into(),
            size: U256::from(2345).into(),
        };
        let sample_new_cap = NewCapability {
            cap: Capability::StoreWrite(sample_cap.clone()),
            parent_index: 0,
        };
        
        let cap_list = NewCapList([sample_new_cap].to_vec()).to_u256_list();

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let new_address = contract.get_proc_addr(String::from("FOO"));
        let new_index = contract.get_proc_index(String::from("FOO"));
        let new_len = contract.get_proc_list_len();
        let hasFoo = contract.contains(String::from("FOO"));

        let new_cap_list_len = contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_STORE_WRITE));
        let new_cap: StoreWriteCap = {
            let raw_cap = contract.get_proc_cap(String::from("FOO"), U256::from(CAP_STORE_WRITE), U256::zero());
            let cap = Capability::from_u256_list(&raw_cap).expect("Should be Valid StoreWriteCap");
            if let Capability::StoreWrite(write_cap) = cap {
                write_cap
            } else {
                panic!("Should be a StoreWrite Cap")
            }
        };

        // Get Id and Truncate
        let mut new_proc_id = contract.get_proc_id(new_index);
        new_proc_id.truncate(3);

        assert_eq!(proc_address, new_address);
        assert_ne!(new_len, U256::zero());
        assert_eq!(new_len.as_u32(), 1);
        assert_eq!(new_len, new_index);
        assert_eq!(new_proc_id, String::from("FOO"));
        assert!(hasFoo);
        assert_eq!(new_cap_list_len, U256::one());
        assert_eq!(new_cap, sample_cap);
    }

    #[test]
    fn should_remove_proc_by_key() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let cap_list = {
            let sample_cap = StoreWriteCap {
                location: U256::from(1234).into(),
                size: U256::from(2345).into(),
            };
            let sample_new_cap = NewCapability {
                cap: Capability::StoreWrite(sample_cap),
                parent_index: 0,
            };
            NewCapList([sample_new_cap].to_vec()).to_u256_list()
        };

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let new_address = contract.get_proc_addr(String::from("FOO"));
        let new_len = contract.get_proc_list_len();
        let new_cap_len = contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_STORE_WRITE));

        assert_eq!(proc_address, new_address);
        assert_ne!(new_len, U256::zero());
        assert_eq!(new_len.as_u32(), 1);
        assert_eq!(new_cap_len, U256::one());

        contract.remove_proc(String::from("FOO"));

        let removed_address = contract.get_proc_addr(String::from("FOO"));
        let removed_index = contract.get_proc_index(String::from("FOO"));
        let removed_len = contract.get_proc_list_len();
        let hasFoo = contract.contains(String::from("FOO"));
        let removed_cap_len = contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_STORE_WRITE));

        assert_eq!(removed_address, H160::zero());
        assert_eq!(removed_index, U256::zero());
        assert_eq!(removed_len, U256::zero());
        assert!(!hasFoo);
        assert_eq!(removed_cap_len, U256::zero());
    }

    #[test]
    fn should_get_proc_cap_list_len() {
        unimplemented!()
    }

    #[test]
    fn should_get_proc_cap() {
        unimplemented!()
    }

    #[test]
    fn should_get_entry_proc_id() {
        unimplemented!()
    }

}
