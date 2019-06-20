extern crate pwasm_abi;
extern crate pwasm_ethereum;
extern crate pwasm_std;


use validator::Cursor;
use validator::serialization::{Serialize, Deserialize, SerializeU256, DeserializeU256};

use pwasm_abi::eth;
use pwasm_abi::types::*;

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
const KERNEL_ENTRY_PROC_PTR: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

pub type ProcedureKey = [u8; 24];
pub type ProcedureIndex = [u8; 24];

pub mod cap;
use cap::*;

use crate::syscalls::*;

pub struct ProcPointer(ProcedureKey);

impl ProcPointer {
    fn from_key(key: ProcedureKey) -> ProcPointer {
        ProcPointer(key)
    }

    /// Get Procedure Storage Pointer
    fn get_store_ptr(&self) -> [u8; 32] {
        let mut result: [u8; 32] = KERNEL_PROC_HEAP_PTR;
        result[5..29].copy_from_slice(&self.0);
        result
    }

    /// Get Procedure Address Pointer
    ///
    /// (Equivalent to crate::get_store_ptr)
    fn get_addr_ptr(&self) -> [u8; 32] {
        self.get_store_ptr()
    }

    fn get_index_ptr(&self) -> [u8; 32] {
        let mut pointer = self.get_store_ptr();
        pointer[31] = 1;
        pointer
    }

    /// Get the Storage Pointer to the Length of a Capability Type List
    fn get_cap_type_len_ptr(&self, cap_type: u8) -> [u8; 32] {
        let mut pointer = self.get_store_ptr();
        pointer[29] = cap_type;
        pointer
    }

    /// Get the Storage Pointer to the Index of a Capability
    fn get_cap_index_ptr(&self, cap_type: u8, cap_index: u8) -> [u8; 32] {
        let mut pointer = self.get_store_ptr();
        pointer[29] = cap_type;
        pointer[30] = cap_index + 1;
        pointer
    }

    /// Get the Storage Pointer of a Capability Value at Index
    fn get_cap_val_ptr(&self, cap_type: u8, cap_index: u8, val_index: u8) -> [u8; 32] {
        let mut pointer = self.get_store_ptr();
        pointer[29] = cap_type;
        pointer[30] = cap_index + 1;
        pointer[31] = val_index;
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
#[derive(Debug, Clone)]
pub enum ProcInsertError {
    /// Procedure Id Already Used
    UsedId = 2,
    /// Procedure List length is greater than 255
    ListFull = 3,
}

/// Inserts Procedure into procedure table
pub fn insert_proc(
    key: ProcedureKey,
    address: Address,
    cap_list: cap::NewCapList,
) -> Result<(), ProcInsertError> {
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

    // Use a static array for cap_type len
    let mut proc_type_len = [0u8; 10];
    let cap_list = cap_list.inner();

    for new_cap in cap_list.iter() {
        let raw_val = new_cap.cap.into_u256_list();
        let cap_type = raw_val[0].as_u32() as u8;

        for (i, val) in raw_val[1..].iter().enumerate() {
            let cap_index = proc_type_len[cap_type as usize];
            pwasm_ethereum::write(
                &H256(proc_pointer.get_cap_val_ptr(cap_type, cap_index, i as u8)),
                &(*val).into(),
            );
        }
        proc_type_len[cap_type as usize] += 1;
    }

    if cap_list.len() > 0 {
        // Update Proc Type Length
        for (cap_type, total_len) in proc_type_len.into_iter().enumerate() {
            if cap_type < 3 {
                continue;
            }
            pwasm_ethereum::write(
                &H256(proc_pointer.get_cap_type_len_ptr(cap_type as u8)),
                &U256::from(*total_len).into(),
            )
        }
    }

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

    // Remove CapList
    for cap_type in 3..10 {
        let cap_type_len =
            pwasm_ethereum::read(&H256(proc_pointer.get_cap_type_len_ptr(cap_type)))[31];
        if cap_type_len == 0 {
            continue;
        };

        let cap_size = match cap_type {
            CAP_PROC_CALL => CAP_PROC_CALL_SIZE,
            CAP_PROC_REGISTER => CAP_PROC_REGISTER_SIZE,
            CAP_PROC_DELETE => CAP_PROC_DELETE_SIZE,
            CAP_PROC_ENTRY => CAP_PROC_ENTRY_SIZE,
            CAP_STORE_WRITE => CAP_STORE_WRITE_SIZE,
            CAP_LOG => CAP_LOG_SIZE,
            CAP_ACC_CALL => CAP_ACC_CALL_SIZE,
            _ => unreachable!(),
        };

        // Remove Each Cap
        for cap_index in 0..cap_type_len {
            for val_index in 0..cap_size {
                let val_pointer = proc_pointer.get_cap_val_ptr(cap_type, cap_index, val_index);
                pwasm_ethereum::write(&H256(val_pointer), &[0u8; 32]);
            }
        }

        // Zero Cap Len
        pwasm_ethereum::write(
            &H256(proc_pointer.get_cap_type_len_ptr(cap_type)),
            &[0u8; 32],
        );
    }

    // Remove Address
    pwasm_ethereum::write(&H256(proc_pointer.get_addr_ptr()), &[0; 32]);

    // Remove Index
    pwasm_ethereum::write(&H256(proc_pointer.get_index_ptr()), &[0; 32]);

    Ok(())
}

#[derive(Debug, Clone)]
pub struct InvalidProcId;

/// Set Entry Procedure Id
pub fn set_entry_proc_id(key: ProcedureKey) -> Result<(), InvalidProcId> {
    if key == [0; 24] {return Err(InvalidProcId);}
    let mut result = [0u8; 32];
    result[8..].copy_from_slice(&key);
    pwasm_ethereum::write(&H256(KERNEL_ENTRY_PROC_PTR), &result);
    Ok(())
}

/// Set Current Procedure Id
pub fn set_current_proc_id(key: ProcedureKey) -> Result<(), InvalidProcId> {
    // Sometime we want to set this value to null.
    // if key == [0; 24] {return Err(InvalidProcId);}
    let mut result = [0u8; 32];
    result[8..].copy_from_slice(&key);
    pwasm_ethereum::write(&H256(KERNEL_CURRENT_PROC_PTR), &result);
    Ok(())
}


pub fn contains(key: ProcedureKey) -> bool {
    // Get Procedure Storage
    let proc_pointer = ProcPointer::from_key(key);

    // Check Procedure Index
    // If Index Is Greater than zero the procedure already exists
    let proc_index = pwasm_ethereum::read(&H256(proc_pointer.get_index_ptr()));
    proc_index[31] != 0
}

/// Get Procedure Address By Key
pub fn get_proc_addr(key: ProcedureKey) -> Option<Address> {
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
pub fn get_proc_index(key: ProcedureKey) -> Option<ProcedureIndex> {
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
pub fn get_proc_id(index: ProcedureIndex) -> Option<ProcedureKey> {
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
pub fn get_proc_list_len() -> U256 {
    // Check Procedure List Length, it must be less than 8^24
    let proc_list_len = pwasm_ethereum::read(&H256(KERNEL_PROC_LIST_PTR));
    U256::from(proc_list_len)
}

/// Get Procedure Cap Type Length
pub fn get_proc_cap_list_len(key: ProcedureKey, cap_type: u8) -> u8 {
    let proc_pointer = ProcPointer::from_key(key);
    let proc_cap_list_len =
        pwasm_ethereum::read(&H256(proc_pointer.get_cap_type_len_ptr(cap_type)));
    proc_cap_list_len[31]
}

/// Get Procedure Capability by Id, Type and Index
pub fn get_proc_cap(key: ProcedureKey, cap_type: u8, cap_index: u8) -> Option<cap::Capability> {
    use cap::*;
    let proc_pointer = ProcPointer::from_key(key);

    let cap_size = match cap_type {
        CAP_PROC_CALL => CAP_PROC_CALL_SIZE,
        CAP_PROC_REGISTER => CAP_PROC_REGISTER_SIZE,
        CAP_PROC_DELETE => CAP_PROC_DELETE_SIZE,
        CAP_PROC_ENTRY => CAP_PROC_ENTRY_SIZE,
        CAP_STORE_WRITE => CAP_STORE_WRITE_SIZE,
        CAP_LOG => CAP_LOG_SIZE,
        CAP_ACC_CALL => CAP_ACC_CALL_SIZE,
        _ => return None,
    };

    let n_caps = get_proc_cap_list_len(key, cap_type);

    if cap_index >= n_caps {
        return None;
    }

    let raw_val: Vec<U256> = (0..cap_size)
        .map(|i| {
            U256::from(pwasm_ethereum::read(&H256(
                proc_pointer.get_cap_val_ptr(cap_type, cap_index, i),
            )))
        })
        .collect();
    let mut cursor = Cursor::new(raw_val.as_slice());
    Some(match cap_type {
        CAP_PROC_CALL => Capability::ProcedureCall(ProcedureCallCap::deserialize_u256(&mut cursor).unwrap()),
        CAP_PROC_REGISTER => Capability::ProcedureRegister(ProcedureRegisterCap::deserialize_u256(&mut cursor).unwrap()),
        CAP_PROC_DELETE => Capability::ProcedureDelete(ProcedureDeleteCap::deserialize_u256(&mut cursor).unwrap()),
        CAP_PROC_ENTRY => Capability::ProcedureEntry(ProcedureEntryCap::deserialize_u256(&mut cursor).unwrap()),
        CAP_STORE_WRITE => Capability::StoreWrite(StoreWriteCap::deserialize_u256(&mut cursor).unwrap()),
        CAP_LOG => Capability::Log(LogCap::deserialize_u256(&mut cursor).unwrap()),
        CAP_ACC_CALL => Capability::AccountCall(AccountCallCap::deserialize_u256(&mut cursor).unwrap()),
        _ => return None,
    })
}

/// Get Entry Procedure Id
pub fn get_entry_proc_id() -> ProcedureKey {
    let proc_id = pwasm_ethereum::read(&H256(KERNEL_ENTRY_PROC_PTR));
    let mut result = [0; 24];
    result.copy_from_slice(&proc_id[8..]);
    result
}

/// Get Current Procedure Id
pub fn get_current_proc_id() -> ProcedureKey {
    let proc_id = pwasm_ethereum::read(&H256(KERNEL_CURRENT_PROC_PTR));
    let mut result = [0; 24];
    result.copy_from_slice(&proc_id[8..]);
    result
}

/// Given a syscall, get the relevant Capability for the current procedure.
pub fn get_cap(syscall: &SysCall) -> Option<cap::Capability> {
    let current_proc_key = get_current_proc_id();
    get_proc_cap(current_proc_key, syscall.cap_type(), syscall.cap_index)
}


#[cfg(test)]
pub mod contract {
    extern crate pwasm_abi_derive;

    use pwasm_abi_derive::eth_abi;
    use super::*;

    #[eth_abi(ProcedureEndpoint, ProcedureClient)]
    pub trait ProcedureTableInterface {
        /// Insert Procedure By Key
        fn insert_proc(&mut self, key: String, address: Address, cap_list: Vec<U256>) -> U256;

        /// Remove Procedure By Key
        fn remove_proc(&mut self, key: String) -> U256;

        /// Set Entry Procedure Id
        fn set_entry_proc_id(&mut self, key: String) -> U256;

        /// Set Current Procedure Id
        fn set_current_proc_id(&mut self, key: String) -> U256;

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

        /// Get Current Procedure Id
        fn get_current_proc_id(&mut self) -> String;
    }

    pub struct ProcedureTableContract;

    impl ProcedureTableInterface for ProcedureTableContract {
        fn insert_proc(&mut self, key: String, address: Address, cap_list: Vec<U256>) -> U256 {
            let raw_key = {
                let byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            let decoded_cap_list =
                cap::NewCapList::from_u256_list(&cap_list).expect("Caplist should be valid");

            match insert_proc(raw_key, address, decoded_cap_list) {
                Ok(()) => U256::zero(),
                Err(_) => U256::one(),
            }
        }

        fn remove_proc(&mut self, key: String) -> U256 {
            let raw_key = {
                let byte_key = key.as_bytes();
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

        /// Set Entry Procedure Id
        fn set_entry_proc_id(&mut self, key: String) -> U256 {
            let raw_key = {
                let byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            match set_entry_proc_id(raw_key) {
                Err(_) => U256::one(),
                Ok(_) => U256::zero()
            }
        }

        /// Set Current Procedure Id
        fn set_current_proc_id(&mut self, key: String) -> U256 {
            let raw_key = {
                let byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            match set_current_proc_id(raw_key) {
                Err(_) => U256::one(),
                Ok(_) => U256::zero()
            }
        }

        fn contains(&mut self, key: String) -> bool {
            let raw_key = {
                let byte_key = key.as_bytes();
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
                let byte_key = key.as_bytes();
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
                let byte_key = key.as_bytes();
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
            let raw_key = {
                let byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            U256::from(get_proc_cap_list_len(raw_key, cap_type.as_u32() as u8))
        }

        fn get_proc_cap(&mut self, key: String, cap_type: U256, cap_index: U256) -> Vec<U256> {
            let raw_key = {
                let byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };

            let cap_type = cap_type.as_u32() as u8;
            let cap_index = cap_index.as_u32() as u8;

            if let Some(cap) = get_proc_cap(raw_key, cap_type, cap_index) {
                cap.into_u256_list()
            } else {
                Vec::new()
            }
        }

        fn get_entry_proc_id(&mut self) -> String {
            let id = get_entry_proc_id();
            if id != [0u8; 24] {
                unsafe { String::from_utf8_unchecked(id.to_vec()) }
            } else {
                String::new()
            }
        }

        fn get_current_proc_id(&mut self) -> String {
            let id = get_current_proc_id();
            if id != [0u8; 24] {
                unsafe { String::from_utf8_unchecked(id.to_vec()) }
            } else {
                String::new()
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;

    use super::cap::*;
    use super::contract;
    use super::contract::*;

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    #[test]
    fn should_insert_proc_by_key() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();

        let cap_list = NewCapList([].to_vec()).to_u256_list();

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let new_address = contract.get_proc_addr(String::from("FOO"));
        let new_index = contract.get_proc_index(String::from("FOO"));
        let new_len = contract.get_proc_list_len();
        let hasFoo = contract.contains(String::from("FOO"));

        // Get Id and Truncate
        let mut new_proc_id = contract.get_proc_id(new_index);
        new_proc_id.truncate(3);

        assert_eq!(proc_address, new_address);
        assert_ne!(new_len, U256::zero());
        assert_eq!(new_len.as_u32(), 1);
        assert_eq!(new_len, new_index);
        assert_eq!(new_proc_id, String::from("FOO"));
        assert!(hasFoo);

    }

    #[test]
    fn should_remove_proc_by_key() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let cap_list = {
            let sample_cap_1 = NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: U256::from(1234).into(),
                    size: U256::from(2345).into(),
                }),
                parent_index: 0,
            };

            let sample_cap_2 = NewCapability {
                cap: Capability::Log(LogCap {
                    topics: 1,
                    t1: [7u8; 32],
                    t2: [0u8; 32],
                    t3: [0u8; 32],
                    t4: [0u8; 32],
                }),
                parent_index: 1,
            };

            NewCapList([sample_cap_1, sample_cap_2].to_vec()).to_u256_list()
        };

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let new_address = contract.get_proc_addr(String::from("FOO"));
        let new_len = contract.get_proc_list_len();
        let new_write_cap_len =
            contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_STORE_WRITE));
        let new_log_cap_len =
            contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_LOG));

        assert_eq!(proc_address, new_address);
        assert_ne!(new_len, U256::zero());
        assert_eq!(new_len.as_u32(), 1);

        assert_eq!(new_write_cap_len, U256::one());
        assert_eq!(new_log_cap_len, U256::one());

        contract.remove_proc(String::from("FOO"));

        let removed_address = contract.get_proc_addr(String::from("FOO"));
        let removed_index = contract.get_proc_index(String::from("FOO"));
        let removed_len = contract.get_proc_list_len();
        let hasFoo = contract.contains(String::from("FOO"));
        let removed_write_cap_len =
            contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_STORE_WRITE));
        let removed_log_cap_len =
            contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_LOG));

        assert_eq!(removed_address, H160::zero());
        assert_eq!(removed_index, U256::zero());
        assert_eq!(removed_len, U256::zero());
        assert!(!hasFoo);

        assert_eq!(removed_write_cap_len, U256::zero());
        assert_eq!(removed_log_cap_len, U256::zero());
    }

    #[test]
    fn should_get_proc_cap_list_len() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let cap_list = {
            let sample_cap_1 = NewCapability {
                cap: Capability::StoreWrite(StoreWriteCap {
                    location: U256::from(1234).into(),
                    size: U256::from(2345).into(),
                }),
                parent_index: 0,
            };

            let sample_cap_2 = NewCapability {
                cap: Capability::Log(LogCap {
                    topics: 1,
                    t1: [7u8; 32],
                    t2: [0u8; 32],
                    t3: [0u8; 32],
                    t4: [0u8; 32],
                }),
                parent_index: 1,
            };

            NewCapList([sample_cap_1, sample_cap_2].to_vec()).to_u256_list()
        };

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let new_write_cap_len =
            contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_STORE_WRITE));
        let new_log_cap_len =
            contract.get_proc_cap_list_len(String::from("FOO"), U256::from(CAP_LOG));

        assert_eq!(new_write_cap_len, U256::one());
        assert_eq!(new_log_cap_len, U256::one());
    }

    #[test]
    fn should_get_proc_cap() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();
        let sample_write_cap = NewCapability {
            cap: Capability::StoreWrite(StoreWriteCap {
                location: U256::from(1234).into(),
                size: U256::from(2345).into(),
            }),
            parent_index: 0,
        };

        let sample_log_cap = NewCapability {
            cap: Capability::Log(LogCap {
                topics: 1,
                t1: [7u8; 32],
                t2: [0u8; 32],
                t3: [0u8; 32],
                t4: [0u8; 32],
            }),
            parent_index: 1,
        };

        let cap_list = NewCapList([sample_write_cap.clone(), sample_log_cap.clone()].to_vec()).to_u256_list();

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let new_write_cap = {
            let raw_cap = contract.get_proc_cap(
                String::from("FOO"),
                U256::from(CAP_STORE_WRITE),
                U256::zero(),
            );
            Capability::from_u256_list(&raw_cap).expect("Should be Valid StoreWriteCap")
        };


        let new_log_cap = {
            let raw_cap = contract.get_proc_cap(
                String::from("FOO"),
                U256::from(CAP_LOG),
                U256::zero(),
            );
            Capability::from_u256_list(&raw_cap).expect("Should be Valid LogCap")
        };

        assert_eq!(new_write_cap, sample_write_cap.cap);
        assert_eq!(new_log_cap, sample_log_cap.cap);
    }

    #[test]
    fn should_get_entry_proc_id() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();

        let cap_list = NewCapList([].to_vec()).to_u256_list();

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let err = contract.set_entry_proc_id(String::from("FOO"));
        if err != U256::zero() {
            panic!("Should return 0 for success, instead got {}", err)
        }

        let mut new_entry_proc_id = contract.get_entry_proc_id();
        new_entry_proc_id.truncate(3);

        assert_eq!(new_entry_proc_id, String::from("FOO"));
    }

    #[test]
    fn should_get_current_proc_id() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();

        let cap_list = NewCapList([].to_vec()).to_u256_list();

        contract.insert_proc(String::from("FOO"), proc_address, cap_list);

        let err = contract.set_current_proc_id(String::from("FOO"));
        if err != U256::zero() {
            panic!("Should return 0 for success, instead got {}", err)
        }

        let mut new_current_proc_id = contract.get_current_proc_id();
        new_current_proc_id.truncate(3);

        assert_eq!(new_current_proc_id, String::from("FOO"));

        let entry_addr = contract.get_proc_addr(String::from("FOO"));
        assert_eq!(entry_addr, proc_address);
    }
}
