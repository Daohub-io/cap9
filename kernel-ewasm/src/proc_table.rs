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

mod cap {
    use super::*;

    pub const CAP_PROC_CALL: u8 = 3;
    pub const CAP_PROC_CALL_SIZE: u8 = 1;

    pub const CAP_PROC_REGISTER: u8 = 4;
    pub const CAP_PROC_REGISTER_SIZE: u8 = 1;

    pub const CAP_PROC_DELETE: u8 = 5;
    pub const CAP_PROC_DELETE_SIZE: u8 = 1;

    pub const CAP_PROC_ENTRY: u8 = 6;
    pub const CAP_PROC_ENTRY_SIZE: u8 = 0;

    pub const CAP_STORE_WRITE: u8 = 7;
    pub const CAP_STORE_WRITE_SIZE: u8 = 2;

    pub const CAP_LOG: u8 = 8;
    pub const CAP_LOG_SIZE: u8 = 5;

    pub const CAP_ACC_CALL: u8 = 9;
    pub const CAP_ACC_CALL_SIZE: u8 = 1;

    #[derive(Clone, Debug)]
    pub struct ProcedureCallCap {
        pub prefix: u8,
        pub key: ProcedureKey,
    }

    #[derive(Clone, Debug)]
    pub struct ProcedureRegisterCap {
        pub prefix: u8,
        pub key: ProcedureKey,
    }

    #[derive(Clone, Debug)]
    pub struct ProcedureDeleteCap {
        pub prefix: u8,
        pub key: ProcedureKey,
    }

    #[derive(Clone, Debug)]
    pub struct ProcedureEntryCap;

    #[derive(Clone, Debug)]
    pub struct StoreWriteCap {
        pub location: [u8; 32],
        pub size: [u8; 32],
    }

    #[derive(Clone, Debug)]
    pub struct LogCap {
        pub topics: u8,
        pub t1: [u8; 32],
        pub t2: [u8; 32],
        pub t3: [u8; 32],
        pub t4: [u8; 32],
    }

    #[derive(Clone, Debug)]
    pub struct AccountCallCap {
        pub can_call_any: bool,
        pub can_send: bool,
        pub address: Address,
    }

    #[derive(Clone, Debug)]
    pub enum Capability {
        ProcedureCall(ProcedureCallCap),
        ProcedureRegister(ProcedureRegisterCap),
        ProcedureDelete(ProcedureDeleteCap),
        ProcedureEntry(ProcedureEntryCap),
        StoreWrite(StoreWriteCap),
        Log(LogCap),
        AccountCall(AccountCallCap),
    }

    #[derive(Clone, Debug)]
    pub struct NewCapability {
        pub cap: Capability,
        pub parent_index: u8,
    }

    #[derive(Debug)]
    pub struct NewCapList(pub Vec<NewCapability>);

    impl NewCapList {
        /// Create Empty CapList
        pub fn empty() -> NewCapList {
            NewCapList(Vec::new())
        }

        pub fn inner(self) -> Vec<NewCapability> {
            self.0
        }
    }

    #[derive(Clone, Debug)]
    pub enum CapDecodeErr {
        InvalidCapType(u8),
        InvalidCapLen(u8),
    }

    impl NewCapList {
        pub fn to_u256_list(&self) -> Vec<U256> {
            
            // Allocate Vector with Max Cap Size
            let mut res = Vec::with_capacity(self.0.len() * (CAP_LOG_SIZE + 3) as usize);
            
            for new_cap in self.0.iter() {
                let raw_cap_slice = match &new_cap.cap {
                    Capability::ProcedureCall(proc_call_cap) => {
                        let cap_size = U256::from(CAP_PROC_CALL_SIZE + 3);
                        let cap_type = U256::from(CAP_PROC_CALL);
                        let parent_index = U256::from(new_cap.parent_index);

                        let mut res = [0u8; 32];
                        res[0] = proc_call_cap.prefix;
                        res[8..].copy_from_slice(&proc_call_cap.key);

                        [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                    },
                    Capability::ProcedureRegister(proc_register_cap) => {
                        let cap_size = U256::from(CAP_PROC_REGISTER_SIZE + 3);
                        let cap_type = U256::from(CAP_PROC_REGISTER);
                        let parent_index = U256::from(new_cap.parent_index);

                        let mut res = [0u8; 32];
                        res[0] = proc_register_cap.prefix;
                        res[8..].copy_from_slice(&proc_register_cap.key);

                        [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                    },
                    Capability::ProcedureDelete(proc_delete_cap) => {
                        let cap_size = U256::from(CAP_PROC_DELETE_SIZE + 3);
                        let cap_type = U256::from(CAP_PROC_DELETE);
                        let parent_index = U256::from(new_cap.parent_index);

                        let mut res = [0u8; 32];
                        res[0] = proc_delete_cap.prefix;
                        res[8..].copy_from_slice(&proc_delete_cap.key);

                        [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                    },
                    Capability::ProcedureEntry(_) => {
                        let cap_size = U256::from(CAP_PROC_ENTRY_SIZE + 3);
                        let cap_type = U256::from(CAP_PROC_ENTRY);
                        let parent_index = U256::from(new_cap.parent_index);

                        [cap_size, cap_type, parent_index].to_vec()
                    },
                    Capability::StoreWrite(store_write_cap) => {
                        let cap_size = U256::from(CAP_STORE_WRITE_SIZE + 3);
                        let cap_type = U256::from(CAP_STORE_WRITE);
                        let parent_index = U256::from(new_cap.parent_index);

                        [cap_size, cap_type, parent_index, U256::from(store_write_cap.location), U256::from(store_write_cap.size)].to_vec()
                    },
                    Capability::Log(log_cap) => {
                        let cap_size = U256::from(CAP_LOG_SIZE + 3);
                        let cap_type = U256::from(CAP_LOG);
                        let parent_index = U256::from(new_cap.parent_index);

                        let topics_len = U256::from(log_cap.topics);
                        let t1 = U256::from(log_cap.t1);
                        let t2 = U256::from(log_cap.t2);
                        let t3 = U256::from(log_cap.t3);
                        let t4 = U256::from(log_cap.t4);

                        [cap_size, cap_type, parent_index, topics_len, t1, t2, t3, t4].to_vec()
                        
                    },
                    Capability::AccountCall(account_call_cap) => {
                        let cap_size = U256::from(CAP_ACC_CALL_SIZE + 3);
                        let cap_type = U256::from(CAP_ACC_CALL);
                        let parent_index = U256::from(new_cap.parent_index);

                        let mut res = [0u8; 32];
                        res[0] |= if account_call_cap.can_call_any { 0x80 } else { 0 };
                        res[0] |= if account_call_cap.can_send { 0x40 } else { 0 };

                        res[12..].copy_from_slice(account_call_cap.address.as_fixed_bytes());

                        [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                    }
                };

                res.extend_from_slice(&raw_cap_slice);
            }

            res
        }

        pub fn from_u256_list(list: &[U256]) -> Result<Self, CapDecodeErr> {
            let mut result = Vec::new();

            // List Length
            let end = list.len();

            // Set Start
            let mut start = 0;

            while start < end {
                
                // Check List Length
                if end - start < 3 {
                    return Err(CapDecodeErr::InvalidCapLen(start as u8));
                }

                // Get Values
                let cap_size = list[start].byte(0);
                let cap_type = list[start + 1].byte(0);
                let parent_index = list[start + 2].byte(0);

                // Check Cap Size
                if end - start > cap_size as usize {
                    return Err(CapDecodeErr::InvalidCapLen(cap_size));
                }

                // Increment Start
                start += 3;

                let new_cap = match (cap_type, cap_size - 3) {
                    (CAP_PROC_CALL, CAP_PROC_CALL_SIZE) => {
                        let val = list[start];
                        let mut key = [0u8; 24];
                        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                        let proc_call_cap = ProcedureCallCap {
                            prefix: val.byte(0),
                            key: key,
                        };

                        NewCapability {
                            cap: Capability::ProcedureCall(proc_call_cap),
                            parent_index,
                        }
                    },
                    (CAP_PROC_REGISTER, CAP_PROC_REGISTER_SIZE) => {
                        let val = list[start];

                        let mut key = [0u8; 24];
                        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                        let proc_reg_cap = ProcedureRegisterCap {
                            prefix: val.byte(0),
                            key: key,
                        };

                        NewCapability {
                            cap: Capability::ProcedureRegister(proc_reg_cap),
                            parent_index,
                        }
                    },
                    (CAP_PROC_DELETE, CAP_PROC_DELETE_SIZE) => {
                        let val = list[start];

                        let mut key = [0u8; 24];
                        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                        let proc_del_cap = ProcedureDeleteCap {
                            prefix: val.byte(0),
                            key: key,
                        };

                        NewCapability {
                            cap: Capability::ProcedureDelete(proc_del_cap),
                            parent_index,
                        }
                    },
                    (CAP_PROC_ENTRY, CAP_PROC_ENTRY_SIZE) => NewCapability {
                        cap: Capability::ProcedureEntry(ProcedureEntryCap),
                        parent_index,
                    },
                    (CAP_STORE_WRITE, CAP_STORE_WRITE_SIZE) => {
                        let location: [u8; 32] = list[start].into();
                        let size: [u8; 32] = list[start + 1].into();

                        let store_write_cap = StoreWriteCap {
                            location: location,
                            size: size,
                        };

                        NewCapability {
                            cap: Capability::StoreWrite(store_write_cap),
                            parent_index,
                        }
                    }
                    (CAP_LOG, CAP_LOG_SIZE) => {
                        let topics_len: usize = list[start].byte(0) as usize;
                        let mut topics = [[0; 32]; 4];
                        if topics_len != 0 {
                            match topics_len {
                                1...4 => {
                                    for i in 0..topics_len {
                                        topics[i] = list[start + i + 1].into()
                                    }
                                }
                                _ => return Err(CapDecodeErr::InvalidCapLen(topics_len as u8)),
                            }
                        }

                        NewCapability {
                            cap: Capability::Log(LogCap {
                                topics: topics_len as u8,
                                t1: topics[0],
                                t2: topics[1],
                                t3: topics[2],
                                t4: topics[3],
                            }),
                            parent_index,
                        }
                    },
                    (CAP_ACC_CALL, CAP_ACC_CALL_SIZE) => {
                        let val = list[start];

                        let can_call_any = val.bit(0);
                        let can_send = val.bit(1);

                        let mut address = [0u8; 20];
                        address.copy_from_slice(&<[u8; 32]>::from(val)[12..]);
                        let address = H160::from(address);

                        let account_call_cap = AccountCallCap {
                            can_call_any: can_call_any,
                            can_send: can_send,
                            address: address,
                        };

                        NewCapability {
                            cap: Capability::AccountCall(account_call_cap),
                            parent_index,
                        }
                    }
                    _ => return Err(CapDecodeErr::InvalidCapType(cap_type)),
                };

                start += cap_size as usize - 3;
                result.push(new_cap);
            }

            Ok(NewCapList(result))
        }
    }
}

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
pub fn insert_proc(key: ProcedureKey, address: Address) -> Result<(), ProcInsertError> {
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
        fn insert_proc(&mut self, key: String, address: Address) -> U256;

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
        fn get_proc_cap(&mut self, key: String, cap_type: U256, cap_index: U256) -> Vec<u8>;

        /// Get Entry Procedure Id
        fn get_entry_proc_id(&mut self) -> String;
    }

    pub struct ProcedureTableContract;

    impl ProcedureTableInterface for ProcedureTableContract {
        fn insert_proc(&mut self, key: String, address: Address) -> U256 {
            let raw_key = {
                let mut byte_key = key.as_bytes();
                let len = byte_key.len();
                let mut output = [0u8; 24];
                output[..len].copy_from_slice(byte_key);
                output
            };
            match insert_proc(raw_key, address) {
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

        fn get_proc_cap(&mut self, key: String, cap_type: U256, cap_index: U256) -> Vec<u8> {
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

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    #[test]
    fn should_insert_proc_by_key() {
        let mut contract = contract::ProcedureTableContract {};
        let proc_address = Address::from_str("ea674fdde714fd979de3edf0f56aa9716b898ec8").unwrap();

        contract.insert_proc(String::from("FOO"), proc_address);

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

        contract.insert_proc(String::from("FOO"), proc_address);
        let new_address = contract.get_proc_addr(String::from("FOO"));
        let new_len = contract.get_proc_list_len();

        assert_eq!(proc_address, new_address);
        assert_ne!(new_len, U256::zero());
        assert_eq!(new_len.as_u32(), 1);

        contract.remove_proc(String::from("FOO"));

        let removed_address = contract.get_proc_addr(String::from("FOO"));
        let removed_index = contract.get_proc_index(String::from("FOO"));
        let removed_len = contract.get_proc_list_len();
        let hasFoo = contract.contains(String::from("FOO"));

        assert_eq!(removed_address, H160::zero());
        assert_eq!(removed_index, U256::zero());
        assert_eq!(removed_len, U256::zero());
        assert!(!hasFoo)
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

    #[test]
    fn should_encode_cap_list() {
        use super::cap::*;
        use super::*;
        use pwasm_abi::eth;
        use pwasm_abi::eth::AbiType;

        let sample_cap = StoreWriteCap {
            location: U256::from(1234).into(),
            size: U256::from(2345).into(),
        };
        let sample_new_cap = NewCapability {
            cap: Capability::StoreWrite(sample_cap),
            parent_index: 0,
        };

        let ENCODED_SAMPLE_WRITE_CAP: Vec<U256> = [
            U256::from(CAP_STORE_WRITE_SIZE + 3),
            U256::from(CAP_STORE_WRITE),
            U256::from(0),
            U256::from(1234),
            U256::from(2345),
        ]
        .to_vec();

        let mut sink = eth::Sink::new(10 * 256);

        let result = NewCapList([sample_new_cap].to_vec()).to_u256_list();

        assert_eq!(result, ENCODED_SAMPLE_WRITE_CAP);
    }

    #[test]
    fn should_decode_cap_list() {
        use super::cap::*;
        use super::*;
        use pwasm_abi::eth;
        use pwasm_abi::eth::AbiType;

        let SAMPLE_WRITE_CAP: Vec<U256> = [
            U256::from(CAP_STORE_WRITE_SIZE + 3),
            U256::from(CAP_STORE_WRITE),
            U256::from(0),
            U256::from(1234),
            U256::from(2345),
        ].to_vec();

        let cap_list_result = cap::NewCapList::from_u256_list(&SAMPLE_WRITE_CAP)
            .expect("Should decode to a capability");

        let write_cap = match &cap_list_result.0[0].cap {
            cap::Capability::StoreWrite(write_cap) => write_cap,
            cap => panic!("Invalid Cap"),
        };

        // Get Location and Size
        let loc = U256::from(write_cap.location);
        let size = U256::from(write_cap.size);

        assert_eq!(loc, U256::from(1234));
        assert_eq!(size, U256::from(2345));
    }

}
