extern crate pwasm_abi;
extern crate pwasm_ethereum;
extern crate pwasm_std;

use validator::Cursor;
use validator::serialization::{Serialize, Deserialize, SerializeU256, DeserializeU256};
use cap9_core;

use pwasm_abi::eth;
use pwasm_abi::types::*;

use core::convert::TryFrom;

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

/// A list of the cap types which we can use for iterating over all cap types.
pub const CAP_TYPES: [u8; 7] = [
    CAP_PROC_CALL,
    CAP_PROC_REGISTER,
    CAP_PROC_DELETE,
    CAP_PROC_ENTRY,
    CAP_STORE_WRITE,
    CAP_LOG,
    CAP_ACC_CALL
];

type ProcedureKey = [u8; 24];
type ProcedureIndex = [u8; 24];

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureCallCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureRegisterCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureDeleteCap {
    pub prefix: u8,
    pub key: ProcedureKey,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureEntryCap;

#[derive(Clone, Debug, PartialEq)]
pub struct StoreWriteCap {
    pub location: [u8; 32],
    pub size: [u8; 32],
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogCap {
    pub topics: u8,
    pub t1: [u8; 32],
    pub t2: [u8; 32],
    pub t3: [u8; 32],
    pub t4: [u8; 32],
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountCallCap {
    pub can_call_any: bool,
    pub can_send: bool,
    pub address: Address,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Capability {
    ProcedureCall(ProcedureCallCap),
    ProcedureRegister(ProcedureRegisterCap),
    ProcedureDelete(ProcedureDeleteCap),
    ProcedureEntry(ProcedureEntryCap),
    StoreWrite(StoreWriteCap),
    Log(LogCap),
    AccountCall(AccountCallCap),
}

impl Capability {
    #[inline]
    pub fn get_cap_size(&self) -> u8 {
        match self {
            Capability::ProcedureCall(_) => CAP_PROC_CALL_SIZE,
            Capability::ProcedureRegister(_) => CAP_PROC_REGISTER_SIZE,
            Capability::ProcedureDelete(_) => CAP_PROC_DELETE_SIZE,
            Capability::ProcedureEntry(_) => CAP_PROC_ENTRY_SIZE,
            Capability::StoreWrite(_) => CAP_STORE_WRITE_SIZE,
            Capability::Log(_) => CAP_LOG_SIZE,
            Capability::AccountCall(_) => CAP_ACC_CALL_SIZE,
        }
    }

    pub fn cap_type(&self) -> u8 {
        match self {
            Capability::ProcedureCall(_) => CAP_PROC_CALL,
            Capability::ProcedureRegister(_) => CAP_PROC_REGISTER,
            Capability::ProcedureDelete(_) => CAP_PROC_DELETE,
            Capability::ProcedureEntry(_) => CAP_PROC_ENTRY,
            Capability::StoreWrite(_) => CAP_STORE_WRITE,
            Capability::Log(_) => CAP_LOG,
            Capability::AccountCall(_) => CAP_ACC_CALL,
        }
    }

    pub fn is_subset_of(&self, parent_cap: &Capability) -> bool {
        match (self, parent_cap) {
            (Capability::ProcedureCall(cap),Capability::ProcedureCall(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureCall(_),_) => false,

            (Capability::StoreWrite(cap),Capability::StoreWrite(parent)) => cap.is_subset_of(parent),
            (Capability::StoreWrite(_),_) => false,

            (Capability::ProcedureRegister(cap),Capability::ProcedureRegister(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureRegister(_),_) => false,

            (Capability::ProcedureDelete(cap),Capability::ProcedureDelete(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureDelete(_),_) => false,

            (Capability::ProcedureEntry(cap),Capability::ProcedureEntry(parent)) => cap.is_subset_of(parent),
            (Capability::ProcedureEntry(_),_) => false,

            (Capability::Log(cap),Capability::Log(parent)) => cap.is_subset_of(parent),
            (Capability::Log(_),_) => false,

            (Capability::AccountCall(cap),Capability::AccountCall(parent)) => cap.is_subset_of(parent),
            (Capability::AccountCall(_),_) => false,
        }
    }
}

pub trait AsCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool;
}

impl AsCap for StoreWriteCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // Base storage address
        if U256::from_big_endian(&self.location) < U256::from_big_endian(&parent_cap.location) {
            return false;
        }
        // Number of additional storage keys
        if (U256::from_big_endian(&self.location) + U256::from_big_endian(&self.size)) > (U256::from_big_endian(&parent_cap.location) + U256::from_big_endian(&parent_cap.size)) {
            return false;
        }
        true
    }
}

impl AsCap for LogCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // First we check the number of required topics. The number of
        // required topics of the requested cap must be equal to or greater
        // than the number of required topics for the current cap.
        if self.topics < parent_cap.topics {
            return false;
        }
        // Next we check that the topics required by the parent cap are
        // also required by the requested cap.
        if parent_cap.topics >= 1 {
            if parent_cap.t1 != self.t1 {
                return false;
            }
        }
        if parent_cap.topics >= 2 {
            if parent_cap.t2 != self.t2 {
                return false;
            }
        }
        if parent_cap.topics >= 3 {
            if parent_cap.t3 != self.t3 {
                return false;
            }
        }
        if parent_cap.topics >= 4 {
            if parent_cap.t4 != self.t4 {
                return false;
            }
        }
        true
    }
}

impl AsCap for ProcedureEntryCap {
    fn is_subset_of(&self, _parent_cap: &Self) -> bool {
        // All of these caps are identical, therefore any cap of this type is
        // the subset of another,
        true
    }
}

impl AsCap for AccountCallCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // If the requested value of callAny is true, then the parent cap
        // value of callAny must be true.
        if self.can_call_any {
            if !parent_cap.can_call_any {
                return false;
            }
        } else {
            // if the parent_cap value is callAny, we don't care about the value
            // of ethAddress. If the requested value of callAny is false we must
            // check that the addresses are the same
            if !parent_cap.can_call_any {
                // the addresses must match
                if self.address != parent_cap.address {
                    return false;
                }
            }
        }

        // if the requested sendValue flag is true, the parent sendValue flag
        // must also be true.
        if self.can_send && !parent_cap.can_send {
            return false;
        }

        // Othwerwise we can consider it a subset
        true
    }
}


impl AsCap for ProcedureRegisterCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // Check that the prefix of B is >= than the prefix of A.
        if parent_cap.prefix > self.prefix {
            return false;
        }
        // The keys must match
        matching_keys(parent_cap.prefix, &parent_cap.key, &self.key)
    }
}

impl AsCap for ProcedureDeleteCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // Check that the prefix of B is >= than the prefix of A.
        if parent_cap.prefix > self.prefix {
            return false;
        }
        // The keys must match
        matching_keys(parent_cap.prefix, &parent_cap.key, &self.key)
    }
}

impl AsCap for ProcedureCallCap {
    fn is_subset_of(&self, parent_cap: &Self) -> bool {
        // Check that the prefix of B is >= than the prefix of A.
        if parent_cap.prefix > self.prefix {
            return false;
        }
        // The keys must match
        matching_keys(parent_cap.prefix, &parent_cap.key, &self.key)
    }
}

pub fn matching_keys(prefix: u8, required_key: &ProcedureKey, requested_key: &ProcedureKey) -> bool {
    // We only want to keep the first $prefix bits of $key, the
    // rest should be zero. We then XOR this value with the
    // requested proc id and the value should be zero. TODO:
    // consider using the unstable BitVec type. For now we will
    // just a u128 and a u64.
    let mut mask_a_array = [0;16];
    mask_a_array.copy_from_slice(&required_key[0..16]);
    let mut mask_b_array = [0;8];
    mask_b_array.copy_from_slice(&required_key[16..24]);

    let shift_amt: u32 = 192_u8.checked_sub(prefix).unwrap_or(0) as u32;

    let prefix_mask_a: u128 = u128::max_value().checked_shl(shift_amt.checked_sub(64).unwrap_or(0)).unwrap_or(0);
    let prefix_mask_b:u64 = u64::max_value().checked_shl(shift_amt).unwrap_or(0);

    // mask_a + mask_b is the key we are allowed
    let mask_a: u128 = u128::from_be_bytes(mask_a_array) & prefix_mask_a;
    let mask_b: u64 = u64::from_be_bytes(mask_b_array) & prefix_mask_b;

    // This is the key we are requesting but cleared
    let mut req_a_array = [0;16];
    req_a_array.copy_from_slice(&requested_key[0..16]);
    let mut req_b_array = [0;8];
    req_b_array.copy_from_slice(&requested_key[16..24]);
    let req_a: u128 = u128::from_be_bytes(req_a_array) & prefix_mask_a;
    let req_b: u64 = u64::from_be_bytes(req_b_array) & prefix_mask_b;

    return (req_a == mask_a) && (req_b == mask_b);
}

#[derive(Clone, Debug)]
pub enum CapDecodeErr {
    InvalidCapType(u8),
    InvalidCapLen(u8),
}

impl Capability {
    pub fn into_u256_list(&self) -> Vec<U256> {
        match self {
            Capability::ProcedureCall(proc_call_cap) => {
                    let cap_type = U256::from(CAP_PROC_CALL);

                    let mut res = [0u8; 32];
                    res[0] = proc_call_cap.prefix;
                    res[8..].copy_from_slice(&proc_call_cap.key);

                    [cap_type, U256::from(res)].to_vec()
                }
                Capability::ProcedureRegister(proc_register_cap) => {
                    let cap_type = U256::from(CAP_PROC_REGISTER);

                    let mut res = [0u8; 32];
                    res[0] = proc_register_cap.prefix;
                    res[8..].copy_from_slice(&proc_register_cap.key);

                    [cap_type, U256::from(res)].to_vec()
                }
                Capability::ProcedureDelete(proc_delete_cap) => {
                    let cap_type = U256::from(CAP_PROC_DELETE);

                    let mut res = [0u8; 32];
                    res[0] = proc_delete_cap.prefix;
                    res[8..].copy_from_slice(&proc_delete_cap.key);

                    [cap_type, U256::from(res)].to_vec()
                }
                Capability::ProcedureEntry(_) => {
                    let cap_type = U256::from(CAP_PROC_ENTRY);

                    [cap_type].to_vec()
                }
                Capability::StoreWrite(store_write_cap) => {
                    let cap_type = U256::from(CAP_STORE_WRITE);

                    [cap_type, U256::from(store_write_cap.location),U256::from(store_write_cap.size)].to_vec()
                }
                Capability::Log(log_cap) => {
                    let cap_type = U256::from(CAP_LOG);

                    let topics_len = U256::from(log_cap.topics);
                    let t1 = U256::from(log_cap.t1);
                    let t2 = U256::from(log_cap.t2);
                    let t3 = U256::from(log_cap.t3);
                    let t4 = U256::from(log_cap.t4);

                    [cap_type, topics_len, t1, t2, t3, t4].to_vec()
                }
                Capability::AccountCall(account_call_cap) => {
                    let cap_type = U256::from(CAP_ACC_CALL);

                    let mut res = [0u8; 32];
                    res[0] |= if account_call_cap.can_call_any {
                        0x80
                    } else {
                        0
                    };
                    res[0] |= if account_call_cap.can_send { 0x40 } else { 0 };

                    res[12..].copy_from_slice(account_call_cap.address.as_fixed_bytes());

                    [cap_type, U256::from(res)].to_vec()
                }
        }
    }

    // TODO: replace with Deserializer
    pub fn from_u256_list(input: &[U256]) -> Result<Capability, CapDecodeErr> {
        let cap_type = input[0].as_u32() as u8;
        let start = 1;
        let new_cap = match (cap_type, input.len() as u8 - 1) {
            (CAP_PROC_CALL, CAP_PROC_CALL_SIZE) => {
                let val = input[start];
                let mut key = [0u8; 24];
                key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                let proc_call_cap = ProcedureCallCap {
                    prefix: val.byte(31),
                    key: key,
                };

                Capability::ProcedureCall(proc_call_cap)
            }
            (CAP_PROC_REGISTER, CAP_PROC_REGISTER_SIZE) => {
                let val = input[start];

                let mut key = [0u8; 24];
                key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                let proc_reg_cap = ProcedureRegisterCap {
                    prefix: val.byte(31),
                    key: key,
                };

                Capability::ProcedureRegister(proc_reg_cap)
            }
            (CAP_PROC_DELETE, CAP_PROC_DELETE_SIZE) => {
                let val = input[start];

                let mut key = [0u8; 24];
                key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

                let proc_del_cap = ProcedureDeleteCap {
                    prefix: val.byte(31),
                    key: key,
                };

                Capability::ProcedureDelete(proc_del_cap)
            }
            (CAP_PROC_ENTRY, CAP_PROC_ENTRY_SIZE) => {
                Capability::ProcedureEntry(ProcedureEntryCap)
            },
            (CAP_STORE_WRITE, CAP_STORE_WRITE_SIZE) => {
                let location: [u8; 32] = input[start].into();
                let size: [u8; 32] = input[start + 1].into();

                let store_write_cap = StoreWriteCap {
                    location: location,
                    size: size,
                };

                Capability::StoreWrite(store_write_cap)
            }
            (CAP_LOG, CAP_LOG_SIZE) => {
                let topics_len: usize = input[start].byte(0) as usize;
                let mut topics = [[0; 32]; 4];
                if topics_len != 0 {
                    match topics_len {
                        1..=4 => {
                            for i in 0..topics_len {
                                topics[i] = input[start + i + 1].into()
                            }
                        }
                        _ => return Err(CapDecodeErr::InvalidCapLen(topics_len as u8)),
                    }
                }

                Capability::Log(LogCap {
                        topics: topics_len as u8,
                        t1: topics[0],
                        t2: topics[1],
                        t3: topics[2],
                        t4: topics[3],
                    })
            }
            (CAP_ACC_CALL, CAP_ACC_CALL_SIZE) => {
                let val = input[start];
                let account_call_cap = val.into();
                Capability::AccountCall(account_call_cap)
            }
            _ => return Err(CapDecodeErr::InvalidCapType(cap_type)),
        };

        Ok(new_cap)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewCapability {
    pub cap: Capability,
    pub parent_index: u8,
}

#[derive(Clone, Debug, PartialEq)]
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
                }
                Capability::ProcedureRegister(proc_register_cap) => {
                    let cap_size = U256::from(CAP_PROC_REGISTER_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_REGISTER);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] = proc_register_cap.prefix;
                    res[8..].copy_from_slice(&proc_register_cap.key);

                    [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                }
                Capability::ProcedureDelete(proc_delete_cap) => {
                    let cap_size = U256::from(CAP_PROC_DELETE_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_DELETE);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] = proc_delete_cap.prefix;
                    res[8..].copy_from_slice(&proc_delete_cap.key);

                    [cap_size, cap_type, parent_index, U256::from(res)].to_vec()
                }
                Capability::ProcedureEntry(_) => {
                    let cap_size = U256::from(CAP_PROC_ENTRY_SIZE + 3);
                    let cap_type = U256::from(CAP_PROC_ENTRY);
                    let parent_index = U256::from(new_cap.parent_index);

                    [cap_size, cap_type, parent_index].to_vec()
                }
                Capability::StoreWrite(store_write_cap) => {
                    let cap_size = U256::from(CAP_STORE_WRITE_SIZE + 3);
                    let cap_type = U256::from(CAP_STORE_WRITE);
                    let parent_index = U256::from(new_cap.parent_index);

                    [
                        cap_size,
                        cap_type,
                        parent_index,
                        U256::from(store_write_cap.location),
                        U256::from(store_write_cap.size),
                    ]
                    .to_vec()
                }
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
                }
                Capability::AccountCall(account_call_cap) => {
                    let cap_size = U256::from(CAP_ACC_CALL_SIZE + 3);
                    let cap_type = U256::from(CAP_ACC_CALL);
                    let parent_index = U256::from(new_cap.parent_index);

                    let mut res = [0u8; 32];
                    res[0] |= if account_call_cap.can_call_any {
                        0x80
                    } else {
                        0
                    };
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

        let mut cursor = Cursor::new(list);
        let mut result = Vec::new();

        while cursor.remaining() > 0 {
            // Check List Length
            if cursor.remaining() < 3 {
                return Err(CapDecodeErr::InvalidCapLen(cursor.remaining() as u8));
            }
            // Get Values
            let cap_size = u8::deserialize_u256(&mut cursor).unwrap();
            let cap_type = u8::deserialize_u256(&mut cursor).unwrap();
            let parent_index = u8::deserialize_u256(&mut cursor).unwrap();

            // Check Cap Size
            if (cursor.remaining()+3) < cap_size as usize {
                return Err(CapDecodeErr::InvalidCapLen(cap_size));
            }

            // TODO: unchecked subtraction
            let new_cap = match (cap_type, cap_size - 3) {
                (CAP_PROC_CALL, CAP_PROC_CALL_SIZE) => {
                    let proc_call_cap = ProcedureCallCap::deserialize_u256(&mut cursor).unwrap();
                    NewCapability {
                        cap: Capability::ProcedureCall(proc_call_cap),
                        parent_index,
                    }
                }
                (CAP_PROC_REGISTER, CAP_PROC_REGISTER_SIZE) => {
                    let proc_reg_cap = ProcedureRegisterCap::deserialize_u256(&mut cursor).unwrap();
                    NewCapability {
                        cap: Capability::ProcedureRegister(proc_reg_cap),
                        parent_index,
                    }
                }
                (CAP_PROC_DELETE, CAP_PROC_DELETE_SIZE) => {
                    let proc_reg_cap = ProcedureDeleteCap::deserialize_u256(&mut cursor).unwrap();
                    NewCapability {
                        cap: Capability::ProcedureDelete(proc_reg_cap),
                        parent_index,
                    }
                }
                (CAP_PROC_ENTRY, CAP_PROC_ENTRY_SIZE) => NewCapability {
                    cap: Capability::ProcedureEntry(ProcedureEntryCap),
                    parent_index,
                },
                (CAP_STORE_WRITE, CAP_STORE_WRITE_SIZE) => {
                    let store_write_cap = StoreWriteCap::deserialize_u256(&mut cursor).unwrap();
                    NewCapability {
                        cap: Capability::StoreWrite(store_write_cap),
                        parent_index,
                    }
                }
                (CAP_LOG, CAP_LOG_SIZE) => {
                    let log_cap = LogCap::deserialize_u256(&mut cursor).unwrap();
                    NewCapability {
                        cap: Capability::Log(log_cap),
                        parent_index,
                    }
                }
                (CAP_ACC_CALL, CAP_ACC_CALL_SIZE) => {
                    let account_call_cap = AccountCallCap::deserialize_u256(&mut cursor).unwrap();
                    NewCapability {
                        cap: Capability::AccountCall(account_call_cap),
                        parent_index,
                    }
                }
                _ => return Err(CapDecodeErr::InvalidCapType(cap_type)),
            };
            result.push(new_cap);
        }
        Ok(NewCapList(result))
    }
}

impl DeserializeU256 for ProcedureCallCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf).unwrap();
        let val: U256 = buf[0];
        let mut key = [0u8; 24];
        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

        Ok(ProcedureCallCap {
            prefix: val.byte(31),
            key: key,
        })
    }
}

impl DeserializeU256 for ProcedureRegisterCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf).unwrap();
        let val: U256 = buf[0];
        let mut key = [0u8; 24];
        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

        Ok(ProcedureRegisterCap {
            prefix: val.byte(31),
            key: key,
        })
    }
}

impl DeserializeU256 for ProcedureDeleteCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf).unwrap();
        let val: U256 = buf[0];
        let mut key = [0u8; 24];
        key.copy_from_slice(&<[u8; 32]>::from(val)[8..]);

        Ok(ProcedureDeleteCap {
            prefix: val.byte(31),
            key: key,
        })
    }
}

impl DeserializeU256 for ProcedureEntryCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(_reader: &mut R) -> Result<Self, Self::Error> {
        Ok(ProcedureEntryCap {})
    }
}

impl DeserializeU256 for StoreWriteCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 2];
        reader.read(&mut buf).unwrap();
        let location: [u8; 32] = buf[0].into();
        let size: [u8; 32] = buf[1].into();

        Ok(StoreWriteCap {
            location: location,
            size: size,
        })
    }
}

impl DeserializeU256 for LogCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 5];
        reader.read(&mut buf).unwrap();

        let topics_len: usize = buf[0].byte(0) as usize;
        let mut topics = [[0; 32]; 4];
        if topics_len != 0 {
            match topics_len {
                1..=4 => {
                    for i in 0..topics_len {
                        topics[i] = buf[i+1].into()
                    }
                }
                _ => return Err(cap9_core::Error::InvalidData)
                // _ => return Err(CapDecodeErr::InvalidCapLen(topics_len as u8)),
            }
        }

        Ok(LogCap {
            topics: topics_len as u8,
            t1: topics[0],
            t2: topics[1],
            t3: topics[2],
            t4: topics[3],
        })
    }
}


impl DeserializeU256 for AccountCallCap {
    type Error = cap9_core::Error;

    fn deserialize_u256<R: cap9_core::Read<U256>>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut buf = [U256::zero(); 1];
        reader.read(&mut buf).unwrap();
        let val: U256 = buf[0];

        let can_call_any = val.bit(255);
        let can_send = val.bit(254);

        let mut address = [0u8; 20];
        address.copy_from_slice(&<[u8; 32]>::from(val)[12..]);
        let address = H160::from(address);

        let account_call_cap = AccountCallCap {
            can_call_any: can_call_any,
            can_send: can_send,
            address: address,
        };

        Ok(account_call_cap)
    }
}

impl From<U256> for AccountCallCap {
    fn from(val: U256) -> Self {
        let can_call_any = val.bit(255);
        let can_send = val.bit(254);

        let mut address = [0u8; 20];
        address.copy_from_slice(&<[u8; 32]>::from(val)[12..]);
        let address = H160::from(address);

        AccountCallCap {
            can_call_any: can_call_any,
            can_send: can_send,
            address: address,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate pwasm_test;

    use self::pwasm_test::{ext_get, ext_reset};
    use core::str::FromStr;
    use pwasm_abi::types::*;

    use super::*;

    use pwasm_abi::eth;
    use pwasm_abi::eth::AbiType;

    #[test]
    fn should_encode_empty_cap_list() {
        let EMPTY_LIST = Vec::new();
        let result = NewCapList(EMPTY_LIST).to_u256_list();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_decode_empty_cap_list() {
        let result = NewCapList::from_u256_list(&Vec::new()).expect("An Empty CapList is a Valid List");
        assert_eq!(result.inner().len(), 0);
    }

    #[test]
    fn should_encode_cap_list() {

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

        let result = NewCapList([sample_new_cap].to_vec()).to_u256_list();

        assert_eq!(result, ENCODED_SAMPLE_WRITE_CAP);
    }

    #[test]
    fn should_decode_cap_list() {

        let SAMPLE_WRITE_CAP: Vec<U256> = [
            U256::from(CAP_STORE_WRITE_SIZE + 3),
            U256::from(CAP_STORE_WRITE),
            U256::from(0),
            U256::from(1234),
            U256::from(2345),
        ]
        .to_vec();

        let cap_list_result = NewCapList::from_u256_list(&SAMPLE_WRITE_CAP)
            .expect("Should decode to a capability");

        let write_cap = match &cap_list_result.0[0].cap {
            Capability::StoreWrite(write_cap) => write_cap,
            _ => panic!("Invalid Cap"),
        };

        // Get Location and Size
        let loc = U256::from(write_cap.location);
        let size = U256::from(write_cap.size);

        assert_eq!(loc, U256::from(1234));
        assert_eq!(size, U256::from(2345));
    }

    #[test]
    fn should_decode_call_cap() {

        let mut arr = [0; 32];
        arr[0] = 3;

        let sample_cap = ProcedureCallCap {
            prefix: 3,
            key: [1; 24]
        };
        let sample_new_cap = NewCapability {
            cap: Capability::ProcedureCall(sample_cap),
            parent_index: 0,
        };

        let input = NewCapList([sample_new_cap].to_vec());
        let encoded = input.to_u256_list();

        let decoded = NewCapList::from_u256_list(&encoded).expect("Should decode call cap");

        assert_eq!(input.inner(), decoded.inner());
    }

    #[test]
    fn should_decode_encode_call_cap() {

        let prefix: u8 = 3;
        // let key = [0x11; 24];
        let key: [u8; 24] = [0x1,0x3,0x4,0x5,0x6,0x7,0x8,0x9,0xa,0xb,0xc,0xd,0xe,0xf,0x10,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1a];
        let mut arr = [0; 32];
        arr[0] = 3;
        arr[8..32].copy_from_slice(&key);

        let sample_cap = Capability::ProcedureCall(ProcedureCallCap {
            prefix,
            key,
        });

        let list = [CAP_PROC_CALL.into(),arr.into()].to_vec();

        assert_eq!(sample_cap.into_u256_list(), list);
        assert_eq!(Capability::from_u256_list(&list).unwrap(), sample_cap);
    }

    #[test]
    fn should_decode_encode_account_call_cap() {

        let raw: [u8;32] = [0xc0,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0x00];
        let raw_address: [u8;20] = [0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0x00];

        let val: U256 = raw.into();
        let address: Address = raw_address.into();
        let cap: AccountCallCap = val.into();
        let expected_cap = AccountCallCap {
            can_send: true,
            can_call_any: true,
            address: address,
        };
        assert_eq!(cap, expected_cap);
    }

}
